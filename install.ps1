$ErrorActionPreference = "Stop"
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12

Write-Host "Installing fyr for Windows..." -ForegroundColor Green

try {
    $webClient = New-Object System.Net.WebClient
    $webClient.Headers.Add("User-Agent", "PowerShell")

    $apiResponse = $webClient.DownloadString("https://api.github.com/repos/opmr0/fyr/releases/latest")
    $version = ($apiResponse | ConvertFrom-Json).tag_name

    if (-not $version) { throw "Failed to fetch latest release version" }

    $downloadUrl = "https://github.com/opmr0/fyr/releases/download/$version/fyr-windows-x86_64.exe"
    $tempFile = Join-Path $env:TEMP "fyr.exe"

    Write-Host "Downloading $version..." -ForegroundColor Cyan
    $webClient.DownloadFile($downloadUrl, $tempFile)

    $installDir = Join-Path $env:LOCALAPPDATA "Programs\fyr"
    if (-not (Test-Path $installDir)) {
        New-Item -ItemType Directory -Path $installDir -Force | Out-Null
    }

    $finalPath = Join-Path $installDir "fyr.exe"
    if (Test-Path $finalPath) { Remove-Item $finalPath -Force }
    Move-Item $tempFile $finalPath -Force

    $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if ($userPath -notlike "*$installDir*") {
        [Environment]::SetEnvironmentVariable("Path", $userPath + ";" + $installDir, "User")
        $env:Path = [Environment]::GetEnvironmentVariable("Path", "Machine") + ";" + $userPath + ";" + $installDir
        Write-Host "Added to PATH" -ForegroundColor Green
        Write-Host "Restart your terminal for PATH changes to take effect" -ForegroundColor Yellow
    }

    Write-Host ""
    Write-Host "fyr installed successfully!" -ForegroundColor Green
    Write-Host "Run 'fyr --help' to get started" -ForegroundColor Cyan

} catch {
    Write-Host "Installation failed: $_" -ForegroundColor Red
    Write-Host ""
    Write-Host "Manual installation:" -ForegroundColor Yellow
    Write-Host "1. Go to: https://github.com/opmr0/fyr/releases/latest" -ForegroundColor Yellow
    Write-Host "2. Download: fyr-windows-x86_64.exe" -ForegroundColor Yellow
    Write-Host "3. Rename to fyr.exe and move to a folder in your PATH" -ForegroundColor Yellow
    exit 1
}
