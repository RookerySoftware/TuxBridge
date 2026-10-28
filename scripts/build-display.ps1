# Builds TuxBridge's own display: the viewer (Windows) with cargo, the agent (Linux) inside a WSL
# distro that has Rust, PipeWire's development files and clang. Puts them where the app bundles
# them: src-tauri\bin\easywsl-viewer.exe and src-tauri\linux\easywsl-agent.
#
#   pwsh .\scripts\build-display.ps1 -BuildDistro easywsl-build -BuildUser builder
#
# The agent links against the build distro's C library, so it runs on that release and newer:
# build it on the oldest release it should run on (Ubuntu 24.04, glibc 2.39, for Linux Mint 22).

param(
    [Parameter(Mandatory)] [string] $BuildDistro,
    [Parameter(Mandatory)] [string] $BuildUser
)

$ErrorActionPreference = 'Stop'
$env:WSL_UTF8 = 1
$root = Resolve-Path (Join-Path $PSScriptRoot '..')
$display = Join-Path $root.Path 'display'

Write-Host '==> The viewer (Windows)'
cargo build --release --manifest-path (Join-Path $display 'Cargo.toml') -p easywsl-viewer
if ($LASTEXITCODE) { throw 'building the viewer failed' }
$bin = Join-Path $root 'src-tauri\bin'
New-Item -ItemType Directory -Force $bin | Out-Null
Copy-Item (Join-Path $display 'target\release\easywsl-viewer.exe') $bin -Force

Write-Host "==> The agent (Linux, in $BuildDistro)"
$wslDisplay = '/mnt/' + $display.Substring(0, 1).ToLower() + $display.Substring(2).Replace('\', '/')
$wslOut = '/mnt/' + $root.Path.Substring(0, 1).ToLower() + $root.Path.Substring(2).Replace('\', '/') + '/src-tauri/linux/easywsl-agent'
wsl.exe -d $BuildDistro -u $BuildUser -e bash -lc "cd '$wslDisplay' && CARGO_TARGET_DIR=`$HOME/easywsl-target cargo build --release -p easywsl-agent && cp `$HOME/easywsl-target/release/easywsl-agent '$wslOut'"
if ($LASTEXITCODE) { throw 'building the agent failed' }
Write-Host 'Done: src-tauri\bin\easywsl-viewer.exe, src-tauri\linux\easywsl-agent'
