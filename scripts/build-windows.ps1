$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$dist = Join-Path $root 'dist'
New-Item -ItemType Directory -Force -Path $dist | Out-Null
cargo build --manifest-path (Join-Path $root 'Cargo.toml') --release --locked
Copy-Item (Join-Path $root 'target\release\lion-autoclicker.exe') $dist -Force
Write-Output "Created $dist\lion-autoclicker.exe"
