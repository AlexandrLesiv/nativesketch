$ErrorActionPreference = 'Stop'
& "$PSScriptRoot/build-windows.ps1"
$env:PATH = "$(Join-Path (Split-Path $PSScriptRoot -Parent) 'target/release');$env:PATH"
dotnet run --project apps/windows/InteropCheck -c Release
if ($LASTEXITCODE -ne 0) { throw 'Interop check failed' }
