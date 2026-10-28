# Writes THIRD-PARTY-NOTICES.md: the open-source libraries built into TuxBridge (the app's Rust
# crates for Windows, the display viewer's, and the window's npm packages), with their licences and
# where their source is. Run before a release; the installer ships the file (tauri.conf.json).
#
#   pwsh .\scripts\notices.ps1

$ErrorActionPreference = 'Stop'
$root = Resolve-Path (Join-Path $PSScriptRoot '..')
$rows = @{}

function Add-Crates($manifest) {
    $meta = cargo metadata --format-version 1 --manifest-path $manifest --filter-platform x86_64-pc-windows-msvc | ConvertFrom-Json
    $ours = @($meta.workspace_members)
    foreach ($p in $meta.packages) {
        if ($ours -contains $p.id) { continue }
        $source = if ($p.repository) { $p.repository } else { "https://crates.io/crates/$($p.name)" }
        $rows["$($p.name) $($p.version)"] = "| $($p.name) | $($p.version) | $($p.license) | $source |"
    }
}

Add-Crates (Join-Path $root 'src-tauri\Cargo.toml')
Add-Crates (Join-Path $root 'display\Cargo.toml')
$crates = $rows.Keys | Sort-Object | ForEach-Object { $rows[$_] }

$npm = @()
Push-Location $root
try {
    $licenses = pnpm licenses list --prod --json | ConvertFrom-Json
    foreach ($license in $licenses.PSObject.Properties) {
        foreach ($pkg in $license.Value) {
            $source = if ($pkg.homepage) { $pkg.homepage } else { "https://www.npmjs.com/package/$($pkg.name)" }
            foreach ($v in @($pkg.versions)) { $npm += "| $($pkg.name) | $v | $($license.Name) | $source |" }
        }
    }
} finally { Pop-Location }
$npm = $npm | Sort-Object -Unique

$text = @"
# Third-party notices

TuxBridge, by Rookery Software, is built with the open-source software below. Each is used under
its licence, named here; the full licence texts are in each project's source, at the address
given. Libraries under the Mozilla Public License 2.0 are used unmodified; their source is at the
addresses listed.

TuxBridge also downloads, at your request and from their own projects, software it does not
include: Linux distributions, Docker Engine, Ollama and AI models, and Microsoft's WebView2
runtime. Those come with their own licences.

Linux® is the registered trademark of Linus Torvalds.

## Rust crates ($($crates.Count))

| Crate | Version | Licence | Source |
| --- | --- | --- | --- |
$($crates -join "`n")

## npm packages ($($npm.Count))

| Package | Version | Licence | Source |
| --- | --- | --- | --- |
$($npm -join "`n")
"@
Set-Content -Path (Join-Path $root 'THIRD-PARTY-NOTICES.md') -Value $text -Encoding utf8
Write-Host "THIRD-PARTY-NOTICES.md: $($crates.Count) crates, $($npm.Count) npm packages"
