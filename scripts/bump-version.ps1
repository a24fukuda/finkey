<#
.SYNOPSIS
    バージョン更新スクリプト
.DESCRIPTION
    バージョン更新、CHANGELOG生成、コミット、PR作成を行う
.PARAMETER Version
    新しいバージョン (例: 0.4.11)
.PARAMETER Force
    PR作成の確認をスキップ
.EXAMPLE
    ./scripts/bump-version.ps1 0.4.11
.EXAMPLE
    ./scripts/bump-version.ps1 0.4.11 -Force
#>

param(
    [Parameter(Mandatory=$true)]
    [string]$Version,

    [switch]$Force
)

$ErrorActionPreference = "Stop"

# 色付き出力
function Write-Step { param($msg) Write-Host "`n=== $msg ===" -ForegroundColor Cyan }
function Write-Success { param($msg) Write-Host $msg -ForegroundColor Green }
function Write-Warning { param($msg) Write-Host $msg -ForegroundColor Yellow }
function Write-Error { param($msg) Write-Host $msg -ForegroundColor Red }

# バージョン形式チェック
if ($Version -notmatch '^\d+\.\d+\.\d+$') {
    Write-Error "Invalid version format: $Version (expected: x.y.z)"
    exit 1
}

# タグ存在チェック
Write-Step "Checking tag v$Version"
$existingTag = git tag -l "v$Version"
if ($existingTag) {
    Write-Error "Tag v$Version already exists"
    exit 1
}
Write-Success "Tag v$Version does not exist"

# ブランチチェック
Write-Step "Checking branch"
$currentBranch = git branch --show-current
if ($currentBranch -eq "main") {
    Write-Host "Creating branch chore/bump-version-$Version..."
    git checkout -b "chore/bump-version-$Version"
} else {
    Write-Warning "Current branch: $currentBranch"
}

# 現在のバージョン取得
$packageJson = Get-Content "package.json" -Raw | ConvertFrom-Json
$currentVersion = $packageJson.version
Write-Host "Current version: $currentVersion"

# package.json 更新
Write-Step "Updating package.json"
$packageJsonContent = Get-Content "package.json" -Raw
$packageJsonContent = $packageJsonContent -replace "`"version`": `"$currentVersion`"", "`"version`": `"$Version`""
Set-Content "package.json" $packageJsonContent -NoNewline
Write-Success "Updated package.json to $Version"

# Cargo.toml 更新
Write-Step "Updating Cargo.toml"
$cargoPath = "src-tauri/Cargo.toml"
$cargoContent = Get-Content $cargoPath -Raw
$cargoContent = $cargoContent -replace "version = `"$currentVersion`"", "version = `"$Version`""
Set-Content $cargoPath $cargoContent -NoNewline
Write-Success "Updated Cargo.toml to $Version"

# CHANGELOG生成
Write-Step "Generating CHANGELOG entry"

# 前回のタグを取得
$lastTag = git describe --tags --abbrev=0 2>$null
if (-not $lastTag) {
    $lastTag = "HEAD~10"
    Write-Warning "No previous tag found, using last 10 commits"
}

Write-Host "Commits since $lastTag..."

# コミットを取得してカテゴリ分け
$commits = git log "$lastTag..HEAD" --format="%s" --no-merges

$added = @()
$changed = @()
$fixed = @()
$docs = @()
$dx = @()

foreach ($commit in $commits) {
    # Claude Code自動生成コミットはスキップ
    if ($commit -match "^chore: bump version") { continue }

    if ($commit -match "^feat:\s*(.+)") {
        $added += "- $($Matches[1])"
    }
    elseif ($commit -match "^fix:\s*(.+)") {
        $fixed += "- $($Matches[1])"
    }
    elseif ($commit -match "^refactor:\s*(.+)") {
        $changed += "- $($Matches[1])"
    }
    elseif ($commit -match "^docs:\s*(.+)") {
        $docs += "- $($Matches[1])"
    }
    elseif ($commit -match "^chore:\s*(.+)") {
        $dx += "- $($Matches[1])"
    }
}

# CHANGELOGエントリ生成
$date = Get-Date -Format "yyyy-MM-dd"
$entry = "## [$Version] - $date`n"

if ($added.Count -gt 0) {
    $entry += "`n### Added`n`n"
    $entry += ($added -join "`n") + "`n"
}
if ($changed.Count -gt 0) {
    $entry += "`n### Changed`n`n"
    $entry += ($changed -join "`n") + "`n"
}
if ($fixed.Count -gt 0) {
    $entry += "`n### Fixed`n`n"
    $entry += ($fixed -join "`n") + "`n"
}
if ($docs.Count -gt 0) {
    $entry += "`n### Documentation`n`n"
    $entry += ($docs -join "`n") + "`n"
}
if ($dx.Count -gt 0) {
    $entry += "`n### Developer Experience`n`n"
    $entry += ($dx -join "`n") + "`n"
}

# エントリがない場合
if ($added.Count -eq 0 -and $changed.Count -eq 0 -and $fixed.Count -eq 0 -and $docs.Count -eq 0 -and $dx.Count -eq 0) {
    Write-Warning "No categorizable commits found"
    $entry += "`n### Changed`n`n- Minor updates`n"
}

Write-Host "`nGenerated entry:"
Write-Host $entry

# CHANGELOG.md 更新
Write-Step "Updating CHANGELOG.md"
$changelog = Get-Content "CHANGELOG.md" -Raw

# ヘッダー部分を分離
$headerPattern = "(?s)^(# Changelog.+?)`n`n## \["
if ($changelog -match $headerPattern) {
    $header = $Matches[1]
    $rest = $changelog.Substring($header.Length + 2)  # +2 for `n`n
    $newChangelog = "$header`n`n$entry`n$rest"
} else {
    Write-Error "Could not parse CHANGELOG.md header"
    exit 1
}

# バージョンリンクを追加
$repoUrl = "https://github.com/a24fukuda/finkey"
$versionLink = "[$Version]: $repoUrl/compare/v$currentVersion...v$Version"
$newChangelog = $newChangelog -replace "(\[$currentVersion\]: .+)", "$versionLink`n`$1"

Set-Content "CHANGELOG.md" $newChangelog -NoNewline
Write-Success "Updated CHANGELOG.md"

# コミット
Write-Step "Committing changes"
git add package.json src-tauri/Cargo.toml CHANGELOG.md
git commit -m "chore: bump version to $Version"
Write-Success "Committed changes"

# プッシュ
Write-Step "Pushing to remote"
$branch = git branch --show-current
git push -u origin $branch
Write-Success "Pushed to origin/$branch"

# PR作成
Write-Step "Creating Pull Request"

# コミット一覧を取得
$commitList = git log "$lastTag..HEAD" --format="- %s" --no-merges | Where-Object { $_ -notmatch "bump version" }

$prBody = @"
## Summary
$($commitList -join "`n")

:robot: Generated with bump-version.ps1
"@

Write-Host "`nPR Title: chore: bump version to $Version"
Write-Host "`nPR Body:"
Write-Host $prBody
Write-Host ""

if (-not $Force) {
    $confirm = Read-Host "Create Pull Request? (y/N)"
    if ($confirm -ne "y") {
        Write-Warning "Cancelled. Changes are committed and pushed."
        Write-Host "To create PR manually: gh pr create"
        exit 0
    }
}

gh pr create --title "chore: bump version to $Version" --body $prBody
Write-Success "Pull Request created"

Write-Host "`n" -NoNewline
Write-Success "Done! Wait for CI to pass, then merge the PR and run ./scripts/release-tag.ps1"
