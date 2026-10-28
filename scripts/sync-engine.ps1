# Copies hyprland-wsl's Linux side (the "engine") into src-tauri/engine, from a given git ref of
# a hyprland-wsl checkout. TuxBridge bundles that copy and runs it inside the distros: the account
# setup, the Hyprland installer, the launcher, Omarchy's installer.
#
#   .\scripts\sync-engine.ps1                                   # pre-release-0.1.0 of P:\hyprland-wsl-experiment
#   .\scripts\sync-engine.ps1 -Repo C:\src\hyprland-wsl -Ref v0.1.0
#
# The ref and its commit are written to src-tauri/engine/VERSION. Commit the result.

param(
    [string] $Repo = 'P:\hyprland-wsl-experiment',
    [string] $Ref = 'pre-release-0.1.0'
)

$ErrorActionPreference = 'Stop'
$engine = Join-Path $PSScriptRoot '..\src-tauri\engine'
$commit = (git -C $Repo rev-parse --verify "$Ref^{commit}").Trim()
if ($LASTEXITCODE) { throw "No ref '$Ref' in $Repo" }

# Only what runs inside Linux. The Windows scripts of hyprland-wsl are replaced by TuxBridge itself.
$paths = @('install.sh', 'hyprland-wsl', 'patches', 'src', 'LICENSE',
           'tools/new-distro-setup.sh', 'tools/merge-layers.sh', 'tools/distro-info.sh',
           'tools/build-aquamarine.sh', 'tools/omarchy-wsl.sh', 'tools/omarchy-wsl-first-start.sh',
           'tools/omarchy-wsl-system.sh', 'tools/omarchy-wsl-user.sh', 'tools/99-hyprland-wsl-omarchy.hook')
$present = @($paths | Where-Object { git -C $Repo cat-file -e "${commit}:$_" 2>$null; -not $LASTEXITCODE })

if (Test-Path $engine) { Remove-Item -Recurse -Force $engine }
New-Item -ItemType Directory $engine | Out-Null
$tar = Join-Path $env:TEMP "easywsl-engine-$commit.tar"
git -C $Repo archive --format=tar -o $tar $commit @present
if ($LASTEXITCODE) { throw 'git archive failed' }
tar -xf $tar -C $engine
Remove-Item $tar
"ref=$Ref`ncommit=$commit`n" | Set-Content -NoNewline -Encoding ascii (Join-Path $engine 'VERSION')
Write-Host "Engine: $Ref ($($commit.Substring(0, 7))), $($present.Count) paths"
