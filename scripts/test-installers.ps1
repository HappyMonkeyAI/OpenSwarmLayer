# Offline boundary tests: downloads and installer process are replaced with fixtures.
$ErrorActionPreference = 'Stop'
$global:OpenSwarmInstallerTestCalled = $false
$global:OpenSwarmInstallerTestDirs = [Collections.Generic.List[string]]::new()
$global:OpenSwarmInstallerTestScenario = ''
function Invoke-WebRequest {
    param($Uri, $OutFile, $TimeoutSec, [switch]$UseBasicParsing)
    $directory = Split-Path -Parent $OutFile
    if (!$global:OpenSwarmInstallerTestDirs.Contains($directory)) { $global:OpenSwarmInstallerTestDirs.Add($directory) }
    if ($global:OpenSwarmInstallerTestScenario -eq 'unavailable') { throw 'Fixture HTTP 404' }
    $asset = 'OpenSwarmLayer-v0.1.1-beta.1-windows-x64-setup.exe'
    $payload = [Text.Encoding]::UTF8.GetBytes('offline installer fixture')
    if ($Uri.EndsWith('/SHA256SUMS')) {
        $hasher = [Security.Cryptography.SHA256]::Create()
        try { $hash = ([BitConverter]::ToString($hasher.ComputeHash($payload))).Replace('-', '').ToLowerInvariant() }
        finally { $hasher.Dispose() }
        if ($global:OpenSwarmInstallerTestScenario -eq 'corrupt') { $hash = '0' * 64 }
        $lines = "$hash  $asset"
        if ($global:OpenSwarmInstallerTestScenario -eq 'missing') { $lines = "$hash  other.exe" }
        if ($global:OpenSwarmInstallerTestScenario -eq 'duplicate') { $lines += "`n$hash  $asset" }
        [IO.File]::WriteAllText($OutFile, $lines)
    } else { [IO.File]::WriteAllBytes($OutFile, $payload) }
}
function Start-Process {
    param($FilePath, [switch]$Wait, [switch]$PassThru)
    $global:OpenSwarmInstallerTestCalled = $true
    if ($global:OpenSwarmInstallerTestScenario -eq 'installer-failure') { return [pscustomobject]@{ ExitCode = 7 } }
    return [pscustomobject]@{ ExitCode = 0 }
}
$installerPath = Join-Path $PSScriptRoot '../install.ps1'
$savedOs = $env:OS
$savedArch = $env:PROCESSOR_ARCHITECTURE
$savedNativeArch = $env:PROCESSOR_ARCHITEW6432
try {
    $env:OS = 'Windows_NT'; $env:PROCESSOR_ARCHITECTURE = 'AMD64'; $env:PROCESSOR_ARCHITEW6432 = ''
    foreach ($case in @('success', 'corrupt', 'missing', 'duplicate', 'unavailable', 'installer-failure', 'download-only')) {
        $global:OpenSwarmInstallerTestScenario = $case; $global:OpenSwarmInstallerTestCalled = $false
        $failed = $false
        try { & $installerPath -DownloadOnly:($case -eq 'download-only') }
        catch { $failed = $true; $failureMessage = $_.Exception.Message }
        $shouldFail = $case -notin @('success', 'download-only')
        if ($failed -ne $shouldFail) { throw "Unexpected result for ${case}: $failureMessage" }
        $shouldInstall = $case -in @('success', 'installer-failure')
        if ($global:OpenSwarmInstallerTestCalled -ne $shouldInstall) { throw "Unexpected installer execution for $case" }
        $latestDir = $global:OpenSwarmInstallerTestDirs[$global:OpenSwarmInstallerTestDirs.Count - 1]
        $remainingPackage = Test-Path -LiteralPath (Join-Path $latestDir 'OpenSwarmLayer-v0.1.1-beta.1-windows-x64-setup.exe')
        if ($remainingPackage -ne ($case -eq 'download-only')) { throw "Unexpected retained package for $case" }
        Write-Output "PASS: $case"
    }
    $global:OpenSwarmInstallerTestCalled = $false
    $env:PROCESSOR_ARCHITECTURE = 'ARM64'
    $failed = $false
    try { & $installerPath } catch { $failed = $true }
    if (!$failed -or $global:OpenSwarmInstallerTestCalled) { throw 'Unsupported architecture was not rejected.' }
    Write-Output 'PASS: unsupported architecture'
    $failed = $false
    try { & $installerPath -Version '../../escape' } catch { $failed = $true }
    if (!$failed) { throw 'Invalid version was accepted.' }
    Write-Output 'PASS: invalid version'
} finally {
    $env:OS = $savedOs; $env:PROCESSOR_ARCHITECTURE = $savedArch; $env:PROCESSOR_ARCHITEW6432 = $savedNativeArch
    foreach ($directory in $global:OpenSwarmInstallerTestDirs) {
        $resolved = [IO.Path]::GetFullPath($directory)
        if (!$resolved.StartsWith([IO.Path]::GetTempPath(), [StringComparison]::OrdinalIgnoreCase) -or (Split-Path -Leaf $resolved) -notmatch '^OpenSwarmLayer-[0-9a-f]{32}$') { throw 'Unexpected fixture directory' }
        foreach ($name in @('SHA256SUMS', 'OpenSwarmLayer-v0.1.1-beta.1-windows-x64-setup.exe')) {
            Remove-Item -LiteralPath (Join-Path $resolved $name) -Force -ErrorAction SilentlyContinue
        }
        if (Test-Path -LiteralPath $resolved) { Remove-Item -LiteralPath $resolved }
    }
    Remove-Variable OpenSwarmInstallerTestCalled,OpenSwarmInstallerTestDirs,OpenSwarmInstallerTestScenario -Scope Global
}
