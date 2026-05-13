# OPTIONAL: Only if you disable the "bundled" feature on sdl2 in Cargo.toml and
# link against MSVC development ZIPs instead. Prepends SDL2 lib\x64 to LIB.
# (This project defaults to bundled SDL2 — you normally do not need this script.)

$sdl2LibX64 = "C:\SDL\SDL2-2.30.0\lib\x64"

if (-not (Test-Path (Join-Path $sdl2LibX64 "SDL2.lib"))) {
    Write-Error "SDL2.lib not found at $sdl2LibX64 — fix sdl2LibX64 (devel-VC zip, lib\x64)."
    exit 1
}

$env:LIB = "$sdl2LibX64;$env:LIB"
Write-Host "LIB is now prefixed with SDL2 x64 lib folder."
Write-Host "Run: cargo build"
