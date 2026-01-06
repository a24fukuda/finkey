param(
    [Parameter(Mandatory=$true)]
    [ValidateSet("on", "off")]
    [string]$Action
)

$ProjectRoot = Split-Path $PSScriptRoot -Parent
$RulesDir = Join-Path $ProjectRoot ".claude\rules"
$EnabledFile = Join-Path $RulesDir "debug-applied-context.md"
$DisabledFile = Join-Path $RulesDir "debug-applied-context.md.disabled"

switch ($Action) {
    "on" {
        if (Test-Path $EnabledFile) {
            Write-Host "Already enabled." -ForegroundColor Yellow
        }
        elseif (Test-Path $DisabledFile) {
            git mv $DisabledFile $EnabledFile
            Write-Host "Enabled debug-applied-context." -ForegroundColor Green
        }
        else {
            Write-Host "Error: File not found." -ForegroundColor Red
            exit 1
        }
    }
    "off" {
        if (Test-Path $DisabledFile) {
            Write-Host "Already disabled." -ForegroundColor Yellow
        }
        elseif (Test-Path $EnabledFile) {
            git mv $EnabledFile $DisabledFile
            Write-Host "Disabled debug-applied-context." -ForegroundColor Green
        }
        else {
            Write-Host "Error: File not found." -ForegroundColor Red
            exit 1
        }
    }
}
