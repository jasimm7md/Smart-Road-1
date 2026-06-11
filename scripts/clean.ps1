# Remove Rust build output and common local temp files.
# Run from anywhere:  .\scripts\clean.ps1
# Or from repo root:   pwsh -File scripts/clean.ps1

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
Set-Location $root

Write-Host "Cleaning Smart-Road (root: $root)"

if (Get-Command cargo -ErrorAction SilentlyContinue) {
    Write-Host "-> cargo clean"
    cargo clean
} else {
    Write-Warning "cargo not found; removing target/ manually."
    if (Test-Path "target") {
        Remove-Item -Recurse -Force "target"
    }
}

$tempPatterns = @(
    @{ Path = "__pycache__"; Type = "Directory" },
    @{ Path = ".pytest_cache"; Type = "Directory" },
    @{ Path = ".DS_Store"; Type = "File" },
    @{ Path = "Thumbs.db"; Type = "File" }
)

foreach ($item in $tempPatterns) {
    if ($item.Type -eq "Directory") {
        Get-ChildItem -Path $root -Recurse -Force -Directory -Filter $item.Path -ErrorAction SilentlyContinue |
            ForEach-Object {
                Write-Host "-> removing $($_.FullName)"
                Remove-Item -Recurse -Force $_.FullName
            }
    } else {
        Get-ChildItem -Path $root -Recurse -Force -File -Filter $item.Path -ErrorAction SilentlyContinue |
            ForEach-Object {
                Write-Host "-> removing $($_.FullName)"
                Remove-Item -Force $_.FullName
            }
    }
}

Get-ChildItem -Path $root -Recurse -Force -File -Filter "*.pyc" -ErrorAction SilentlyContinue |
    ForEach-Object {
        Write-Host "-> removing $($_.FullName)"
        Remove-Item -Force $_.FullName
    }

Write-Host "Done."
