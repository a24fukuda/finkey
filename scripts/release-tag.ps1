<#
.SYNOPSIS
    署名付きバージョンタグを作成してプッシュする

.DESCRIPTION
    package.json からバージョンを取得し、署名付きタグを作成してリモートにプッシュする。
    タグのプッシュにより、GitHub Actions のリリースワークフローが自動実行される。

.EXAMPLE
    ./scripts/release-tag.ps1
#>

$ErrorActionPreference = "Stop"

# バージョン取得
$version = node -p "require('./package.json').version"
$tagName = "v$version"

Write-Host "Release Tag Script" -ForegroundColor Cyan
Write-Host "==================" -ForegroundColor Cyan
Write-Host ""

# タグの存在確認
$existingTag = git tag -l $tagName
if ($existingTag) {
    Write-Host "Error: Tag $tagName already exists." -ForegroundColor Red
    exit 1
}

# 署名設定の確認
$signingKey = git config --get user.signingkey 2>$null

if (-not $signingKey) {
    Write-Host "Warning: No signing key configured." -ForegroundColor Yellow
    Write-Host "See docs/release.md for GPG/SSH signing setup." -ForegroundColor Yellow
    Write-Host ""
}

# 確認
Write-Host "Tag to create: $tagName" -ForegroundColor White
Write-Host "Message: Release $tagName" -ForegroundColor White
Write-Host ""

$confirm = Read-Host "Create and push this tag? (y/N)"
if ($confirm -ne "y" -and $confirm -ne "Y") {
    Write-Host "Cancelled." -ForegroundColor Yellow
    exit 0
}

# タグ作成
Write-Host ""
Write-Host "Creating tag..." -ForegroundColor Cyan
git tag -s $tagName -m "Release $tagName"

if ($LASTEXITCODE -ne 0) {
    Write-Host "Error: Failed to create tag." -ForegroundColor Red
    exit 1
}

Write-Host "Tag created: $tagName" -ForegroundColor Green

# プッシュ
Write-Host ""
Write-Host "Pushing tag..." -ForegroundColor Cyan
git push origin $tagName

if ($LASTEXITCODE -ne 0) {
    Write-Host "Error: Failed to push tag." -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "Done!" -ForegroundColor Green
Write-Host "Tag $tagName has been pushed." -ForegroundColor Green
Write-Host "Release workflow will start automatically." -ForegroundColor Cyan
