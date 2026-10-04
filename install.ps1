# OpenSwarmLayer technical test beta installer. Requires Windows PowerShell 5.1+.
[CmdletBinding()]
param(
    [ValidatePattern('^v[0-9]+\.[0-9]+\.[0-9]+-beta\.[0-9]+$')]
    [string]$Version = 'v0.1.1-beta.1',
    [switch]$DownloadOnly
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
if ($env:OS -ne 'Windows_NT') { throw 'This installer requires Windows.' }
$nativeArch = $env:PROCESSOR_ARCHITEW6432
if (!$nativeArch) { $nativeArch = $env:PROCESSOR_ARCHITECTURE }
if ($nativeArch -ne 'AMD64') { throw 'Only Windows x64 packages are supported.' }

Write-Host "OpenSwarmLayer $Version - technical test beta; not ready for public release."
Write-Host 'The Windows package is unsigned. Close the app before installing or upgrading.'
$asset = "OpenSwarmLayer-$Version-windows-x64-setup.exe"
$baseUrl = "https://github.com/SPhillips1337/OpenSwarmLayer/releases/download/$Version"
$downloadDir = Join-Path ([IO.Path]::GetTempPath()) ('OpenSwarmLayer-' + [guid]::NewGuid().ToString('N'))
$null = New-Item -ItemType Directory -Path $downloadDir
$packagePath = Join-Path $downloadDir $asset
$checksumsPath = Join-Path $downloadDir 'SHA256SUMS'
$verified = $false
# TLS 1.2 is needed by GitHub on older Windows PowerShell installations.
[Net.ServicePointManager]::SecurityProtocol = [Net.ServicePointManager]::SecurityProtocol -bor [Net.SecurityProtocolType]::Tls12
try {
    Invoke-WebRequest -UseBasicParsing -TimeoutSec 600 -Uri "$baseUrl/SHA256SUMS" -OutFile $checksumsPath
    Invoke-WebRequest -UseBasicParsing -TimeoutSec 600 -Uri "$baseUrl/$asset" -OutFile $packagePath
    $expectedHashes = @()
    foreach ($line in Get-Content -LiteralPath $checksumsPath) {
        if ($line -match '^([a-fA-F0-9]{64})  (\S+)$' -and $Matches[2] -ceq $asset) {
            $expectedHashes += $Matches[1]
        }
    }
    if ($expectedHashes.Count -ne 1) { throw 'Missing or duplicate package checksum.' }
    $actualHash = (Get-FileHash -LiteralPath $packagePath -Algorithm SHA256).Hash
    if ($actualHash -ine $expectedHashes[0]) { throw 'Package checksum mismatch; installation refused.' }
    $verified = $true
    Write-Host "SHA-256 verified: $asset"
    if ($DownloadOnly) {
        Write-Host "Verified package saved to $packagePath"
        return
    }
    # Keep the normal installer UI so the user sees installation/upgrade choices.
    $installer = Start-Process -FilePath $packagePath -Wait -PassThru
    if ($installer.ExitCode -ne 0) { throw "Installer exited with code $($installer.ExitCode)." }
    Write-Host 'Installer finished. Launch OpenSwarmLayer to begin your test.'
} catch {
    throw "Beta installation failed. The release may not be published or accessible (the repository is currently private). $($_.Exception.Message)"
} finally {
    # Only remove these two files in our unique temporary directory. No recursive deletion.
    if (!$DownloadOnly -or !$verified) { Remove-Item -LiteralPath $packagePath -Force -ErrorAction SilentlyContinue }
    Remove-Item -LiteralPath $checksumsPath -Force -ErrorAction SilentlyContinue
    if (!(Test-Path -LiteralPath $packagePath)) { Remove-Item -LiteralPath $downloadDir -ErrorAction SilentlyContinue }
}
