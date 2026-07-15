$ErrorActionPreference = "Stop"

$repo = "TAKAMAgents/grok-eliminator"
$installDir = if ($env:GROK_ELIMINATOR_INSTALL_DIR) {
    $env:GROK_ELIMINATOR_INSTALL_DIR
} else {
    Join-Path $HOME ".local\bin"
}

$arch = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture
if ($arch -ne [System.Runtime.InteropServices.Architecture]::X64) {
    throw "unsupported Windows architecture: $arch"
}

$asset = "grok-eliminator-windows-x86_64.zip"
$url = "https://github.com/$repo/releases/latest/download/$asset"
$tmpDir = Join-Path ([System.IO.Path]::GetTempPath()) "grok-eliminator-$([System.Guid]::NewGuid())"
$archive = Join-Path $tmpDir $asset

New-Item -ItemType Directory -Force -Path $tmpDir | Out-Null
New-Item -ItemType Directory -Force -Path $installDir | Out-Null

try {
    Invoke-WebRequest -Uri $url -OutFile $archive
    Expand-Archive -Path $archive -DestinationPath $tmpDir -Force
    Copy-Item (Join-Path $tmpDir "grok-eliminator.exe") (Join-Path $installDir "grok-eliminator.exe") -Force
} finally {
    Remove-Item -Recurse -Force $tmpDir -ErrorAction SilentlyContinue
}

Write-Output "installed: $installDir\grok-eliminator.exe"
Write-Output "run: $installDir\grok-eliminator.exe audit"

$pathEntries = $env:Path -split [System.IO.Path]::PathSeparator
if ($pathEntries -notcontains $installDir) {
    Write-Output "if grok-eliminator.exe is not found, add this directory to PATH:"
    Write-Output $installDir
}
