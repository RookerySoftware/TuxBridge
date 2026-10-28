# Builds a release of TuxBridge: refreshes the third-party notices, then builds the installer and
# signs it for the updater with Rookery Software's private key.
#
#   pwsh .\scripts\release-build.ps1
#
# Makes src-tauri\target\release\bundle\nsis\TuxBridge_<version>_x64-setup.exe and, next to it,
# its signature (.exe.sig). Attach both to the GitHub release: the signature is what lets
# Settings → Update TuxBridge offer the new version.
#
# The private key is %USERPROFILE%\.tauri\tuxbridge-updater.key (or TAURI_SIGNING_PRIVATE_KEY).
# Keep it safe and backed up: without it, existing installs can't be updated, only reinstalled.

$ErrorActionPreference = 'Stop'
$root = Resolve-Path (Join-Path $PSScriptRoot '..')

if (-not $env:TAURI_SIGNING_PRIVATE_KEY) {
    $key = Join-Path $env:USERPROFILE '.tauri\tuxbridge-updater.key'
    if (-not (Test-Path $key)) { throw "No signing key: expected $key (or set TAURI_SIGNING_PRIVATE_KEY)." }
    $env:TAURI_SIGNING_PRIVATE_KEY = Get-Content $key -Raw
}

Push-Location $root
try {
    pwsh -NoProfile -File (Join-Path $PSScriptRoot 'notices.ps1')
    pnpm tauri build
    if ($LASTEXITCODE) { throw "The build failed." }
    $version = (Get-Content src-tauri\tauri.conf.json -Raw | ConvertFrom-Json).version
    $setup = "src-tauri\target\release\bundle\nsis\TuxBridge_${version}_x64-setup.exe"
    if (-not (Test-Path "$setup.sig")) { throw "The installer wasn't signed ($setup.sig is missing)." }
    Write-Host "`nReady: $setup (+ .sig)"
    Write-Host "SHA-256: $((Get-FileHash $setup -Algorithm SHA256).Hash.ToLower())"
} finally { Pop-Location }
