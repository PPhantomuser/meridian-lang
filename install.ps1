$ErrorActionPreference = "Stop"

$Repo = "PPhantomuser/meridian-lang"
$InstallDir = "$HOME\.meridian\bin"
$BinName = "meridian.exe"

Write-Host ""
Write-Host "    __  ___           _     ___"
Write-Host "   /  |/  /__________(_)___/ (_)___ _____ "
Write-Host "  / /|_/ / _ \ ___/ / / __  / / __ \`/ __ \"
Write-Host " / /  / /  __/ /  / / / /_/ / / /_/ / / / /"
Write-Host "/_/  /_/\___/_/  /_/_/\__,_/_/\__,_/_/ /_/ "
Write-Host ""
Write-Host "Welcome to the Meridian installer!"
Write-Host ""

$Architecture = $env:PROCESSOR_ARCHITECTURE.ToLower()
if ($Architecture -eq "amd64" -or $Architecture -eq "x86_64") {
    $Arch = "x86_64"
} else {
    Write-Error "Unsupported architecture: $Architecture"
    exit 1
}

$ReleaseFile = "meridian-windows-${Arch}.exe"
$DownloadUrl = "https://github.com/${Repo}/releases/latest/download/${ReleaseFile}"

Write-Host ">> Downloading Meridian for Windows ($Arch)..."

if (!(Test-Path -Path $InstallDir)) {
    New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
}

$DestPath = Join-Path -Path $InstallDir -ChildPath $BinName

try {
    Invoke-WebRequest -Uri $DownloadUrl -OutFile $DestPath
    Write-Host ">> Successfully downloaded."
} catch {
    Write-Error "Error: Failed to download from $DownloadUrl"
    Write-Host "Make sure the release exists on GitHub!"
    exit 1
}

# Add to PATH
$UserPath = [Environment]::GetEnvironmentVariable("PATH", "User")
if ($UserPath -notmatch [regex]::Escape($InstallDir)) {
    $NewPath = "$InstallDir;$UserPath"
    [Environment]::SetEnvironmentVariable("PATH", $NewPath, "User")
    Write-Host ">> Added $InstallDir to your PATH"
}

Write-Host ""
Write-Host "Meridian was successfully installed!"
Write-Host "To get started, please restart your PowerShell terminal."
Write-Host ""
Write-Host "Then test your installation:"
Write-Host "    meridian --version"
Write-Host ""
