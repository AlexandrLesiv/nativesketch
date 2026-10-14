$ErrorActionPreference = 'Stop'
& "$PSScriptRoot/build-windows.ps1"
$env:PATH = "$(Join-Path (Split-Path $PSScriptRoot -Parent) 'target/x86_64-pc-windows-msvc/release');$env:PATH"
dotnet run --project apps/windows/InteropCheck -c Release -r win-x64
if ($LASTEXITCODE -ne 0) { throw 'Interop check failed' }
