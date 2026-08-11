[CmdletBinding()]
param()

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

$repoRoot = Split-Path -Parent $PSScriptRoot
$source = Join-Path $repoRoot "third_party\fonts\roboto-mono\RobotoMono-wght.ttf"
$output = Join-Path $repoRoot "hal\src\x86_64\roboto-mono-8x16-alpha.bin"

if (-not (Test-Path -LiteralPath $source)) {
    throw "Roboto Mono source font is missing: $source"
}

& cargo run --quiet --package font-rasterizer -- $source $output
if ($LASTEXITCODE -ne 0) { throw "Font atlas generation failed with exit code $LASTEXITCODE." }

$item = Get-Item -LiteralPath $output
if ($item.Length -ne 32768) { throw "Font atlas must be exactly 32768 bytes; got $($item.Length)." }
$hash = (Get-FileHash -Algorithm SHA256 -LiteralPath $output).Hash.ToLowerInvariant()
Write-Host "[SUCCESS] Roboto Mono atlas: $output"
Write-Host "[SUCCESS] SHA-256: $hash"
