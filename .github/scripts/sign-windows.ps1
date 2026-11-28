[CmdletBinding()]
param(
    [Parameter(Mandatory)][string] $Package,
    [Parameter(Mandatory)][string] $Output,
    [Parameter(Mandatory)][ValidatePattern('^v[0-9]+\.[0-9]+\.[0-9]+(?:\+[0-9]+)?$')][string] $Tag,
    [switch] $Authenticode
)

$ErrorActionPreference = 'Stop'
$PSNativeCommandUseErrorActionPreference = $true
$Package = (Resolve-Path -LiteralPath $Package).Path
$requiredFiles = @(
    'carbforge-cli.exe', 'crabforge-cli.exe', 'LICENSE', 'THIRD_PARTY_NOTICES',
    'desktop/crabforge.exe', 'desktop/crabforge_bridge.dll', 'desktop/flutter_windows.dll',
    'desktop/file_selector_windows_plugin.dll', 'desktop/data/app.so',
    'desktop/data/icudtl.dat', 'desktop/data/flutter_assets/AssetManifest.bin'
)
foreach ($required in $requiredFiles) {
    if (!(Test-Path -LiteralPath (Join-Path $Package $required) -PathType Leaf)) {
        throw "Release binary is missing: $required"
    }
}

$files = @(Get-ChildItem -LiteralPath $Package -Recurse -File -Force)
if (Test-Path -LiteralPath (Join-Path $Package 'SHA256SUMS')) {
    throw 'Package already contains a SHA-256 manifest.'
}

function Add-WindowsPublisherSignature {
    $thumbprint = $env:WINDOWS_SIGNING_CERT_THUMBPRINT
    if ([string]::IsNullOrWhiteSpace($env:WINDOWS_SIGNING_PFX_BASE64) -or
        [string]::IsNullOrWhiteSpace($env:WINDOWS_SIGNING_PFX_PASSWORD) -or
        $thumbprint -notmatch '^[0-9A-Fa-f]{40}$') {
        throw 'Authenticode requires WINDOWS_SIGNING_PFX_BASE64, WINDOWS_SIGNING_PFX_PASSWORD, and WINDOWS_SIGNING_CERT_THUMBPRINT.'
    }

    $tools = Join-Path ${env:ProgramFiles(x86)} 'Windows Kits/10/bin/*/x64/signtool.exe'
    $signTool = Get-ChildItem -Path $tools -File |
        Sort-Object { [version]$_.Directory.Parent.Name } -Descending | Select-Object -First 1
    if (!$signTool) { throw 'Windows SDK SignTool is unavailable.' }

    $storePath = "Cert:\CurrentUser\My\$thumbprint"
    if (Test-Path -LiteralPath $storePath) { throw 'The signing certificate is already installed in the runner.' }
    $pfx = Join-Path $env:RUNNER_TEMP ("crabforge-signing-{0}.pfx" -f [Guid]::NewGuid())
    $password = ConvertTo-SecureString $env:WINDOWS_SIGNING_PFX_PASSWORD -AsPlainText -Force
    $pfxBytes = $null
    try {
        $pfxBytes = [Convert]::FromBase64String($env:WINDOWS_SIGNING_PFX_BASE64)
        [IO.File]::WriteAllBytes($pfx, $pfxBytes)
        $endEntities = @((Get-PfxData -FilePath $pfx -Password $password).EndEntityCertificates)
        if ($endEntities.Count -ne 1 -or $endEntities[0].Thumbprint -ne $thumbprint) {
            throw 'The PFX must contain one end-entity certificate matching the configured publisher thumbprint.'
        }
        $certificates = @(Import-PfxCertificate -FilePath $pfx -CertStoreLocation 'Cert:\CurrentUser\My' -Password $password)
        $certificate = $certificates | Where-Object Thumbprint -EQ $thumbprint | Select-Object -First 1
        if (!$certificate -or !$certificate.HasPrivateKey) {
            throw 'The configured publisher certificate and private key were not found in the PFX.'
        }
        Remove-Item -LiteralPath $pfx
        [Array]::Clear($pfxBytes)
        Remove-Item Env:WINDOWS_SIGNING_PFX_BASE64, Env:WINDOWS_SIGNING_PFX_PASSWORD

        foreach ($file in $files | Where-Object Extension -In '.exe', '.dll') {
            $status = Get-AuthenticodeSignature -LiteralPath $file.FullName
            $append = if ($status.SignerCertificate) { @('/as') } else { @() }
            & $signTool.FullName sign @append /sha1 $thumbprint /s My /fd SHA256 `
                /tr http://timestamp.digicert.com /td SHA256 $file.FullName
            & $signTool.FullName verify /pa /all /tw $file.FullName
        }
    } finally {
        if (Test-Path -LiteralPath $pfx) { Remove-Item -LiteralPath $pfx -Force }
        if ($null -ne $pfxBytes) { [Array]::Clear($pfxBytes) }
        $password.Dispose()
        if (Test-Path -LiteralPath $storePath) { Remove-Item -LiteralPath $storePath -DeleteKey }
    }
}

if ($Authenticode) {
    Add-WindowsPublisherSignature
}

$manifest = foreach ($file in $files | Sort-Object FullName -CaseSensitive) {
    $relative = [IO.Path]::GetRelativePath($Package, $file.FullName).Replace('\', '/')
    "{0}  {1}" -f (Get-FileHash -LiteralPath $file.FullName -Algorithm SHA256).Hash.ToLowerInvariant(), $relative
}
$manifest | Set-Content -LiteralPath (Join-Path $Package 'SHA256SUMS') -Encoding utf8NoBOM
New-Item -ItemType Directory -Path $Output | Out-Null
$zip = Join-Path $Output "crabforge-$($Tag.Substring(1))-windows-x64.zip"
[IO.Compression.ZipFile]::CreateFromDirectory($Package, $zip, [IO.Compression.CompressionLevel]::Optimal, $false)
$digest = (Get-FileHash -LiteralPath $zip -Algorithm SHA256).Hash.ToLowerInvariant()
[IO.File]::WriteAllText("$zip.sha256", "$digest  $([IO.Path]::GetFileName($zip))`n")
