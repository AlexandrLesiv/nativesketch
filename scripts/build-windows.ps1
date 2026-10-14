$ErrorActionPreference = 'Stop'
Set-Location (Split-Path $PSScriptRoot -Parent)
if (-not $IsWindows) { throw 'WinUI build requires Windows with .NET 10, Rust MSVC and Windows SDK.' }
cargo build --locked --release -p sketch-ffi --target x86_64-pc-windows-msvc
if ($LASTEXITCODE -ne 0) { throw 'Rust build failed' }
dotnet build apps/windows/NativeSketch.Windows.csproj -c Release
if ($LASTEXITCODE -ne 0) { throw 'WinUI build failed' }
