[CmdletBinding()]
param([Parameter(Mandatory)][string] $Destination)

$ErrorActionPreference = 'Stop'
$PSNativeCommandUseErrorActionPreference = $true
$root = (Resolve-Path (Join-Path $PSScriptRoot '../..')).Path
$target = 'x86_64-pc-windows-msvc'
$cargoOutput = if ($env:CARGO_TARGET_DIR) { $env:CARGO_TARGET_DIR } else { Join-Path $root 'target' }

Push-Location $root
try {
    $desktop = Join-Path $root 'gui/build/windows/x64/runner/Release'
    foreach ($file in 'crabforge.exe', 'crabforge_bridge.dll', 'flutter_windows.dll', 'file_selector_windows_plugin.dll', 'data/app.so') {
        if (!(Test-Path -LiteralPath (Join-Path $desktop $file) -PathType Leaf)) {
            throw "Missing desktop output: $file"
        }
    }
    $logo = Join-Path $desktop 'data/flutter_assets/packages/crabforge_assets/crabforge-mark.png'
    if ((Get-FileHash -LiteralPath $logo).Hash -ne (Get-FileHash -LiteralPath 'assets/crabforge-mark.png').Hash) {
        throw 'The packaged Crabforge logo does not match assets/crabforge-mark.png.'
    }
    New-Item -ItemType Directory -Path $Destination | Out-Null
    $Destination = (Resolve-Path -LiteralPath $Destination).Path
    Copy-Item -LiteralPath $desktop -Destination (Join-Path $Destination 'desktop') -Recurse
    foreach ($name in 'carbforge-cli.exe', 'crabforge-cli.exe') {
        Copy-Item -LiteralPath (Join-Path $cargoOutput "$target/release/$name") -Destination $Destination
    }
    Copy-Item -LiteralPath 'LICENSE', 'THIRD_PARTY_NOTICES' -Destination $Destination

    $metadata = cargo metadata --locked --filter-platform $target --format-version 1 | ConvertFrom-Json
    $graph = cargo tree --locked --target $target --edges normal,build --prefix none --format '{p}' `
        --package crabforge-cli --package crabforge-runner --package crabforge-bridge
    $enabled = [System.Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    foreach ($line in $graph) {
        if ($line -match '^([^ ]+) v([^ ]+)') { [void]$enabled.Add("$($Matches[1])-$($Matches[2])") }
    }
    $licenses = Join-Path $Destination 'licenses/rust'
    foreach ($package in $metadata.packages | Where-Object source -Like 'registry+*') {
        $id = "$($package.name)-$($package.version)"
        if (!$enabled.Contains($id)) { continue }
        $dir = New-Item -ItemType Directory -Path (Join-Path $licenses $id) -Force
        $manifestDir = Split-Path -Parent $package.manifest_path
        $licenseFiles = Get-ChildItem -LiteralPath $manifestDir -File | Where-Object {
            $_.Name -match '^(licen[cs]e|copying|copyright|notice)(?:$|[._-])'
        }
        foreach ($file in $licenseFiles) { Copy-Item -LiteralPath $file.FullName -Destination $dir.FullName }
        if ($package.license_file) {
            Copy-Item -LiteralPath (Join-Path $manifestDir $package.license_file) -Destination $dir.FullName
        }
        if (!$licenseFiles -and !$package.license_file) {
            # Some registry packages declare an SPDX expression without shipping license text.
            "SPDX: $($package.license)`nRepository: $($package.repository)" |
                Set-Content -LiteralPath (Join-Path $dir.FullName 'Cargo-license.txt') -Encoding utf8NoBOM
        }
    }
    Copy-Item -LiteralPath 'licenses/rust-upstream.txt' -Destination (Split-Path -Parent $licenses)

    $catalogs = foreach ($name in 'carbforge-cli.exe', 'crabforge-cli.exe') {
        & (Join-Path $Destination $name) catalog | Out-String
    }
    if ($catalogs[0] -cne $catalogs[1] -or ($catalogs[0] | ConvertFrom-Json).Count -eq 0) {
        throw 'Packaged CLI catalogs are empty or disagree.'
    }
} finally {
    Pop-Location
}
