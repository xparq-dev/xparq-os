$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..\..")
$elfPath = Join-Path $repoRoot "target\x86_64-unknown-none\release\xparq_kernel"
$outPath = Join-Path $repoRoot "build\x86-64\kernel.bin"

if (-not (Test-Path $elfPath)) {
    Push-Location $repoRoot
    cargo build --target x86_64-unknown-none --release --package xparq-kernel
    Pop-Location
}
if (-not (Test-Path $elfPath)) {
    throw "Kernel ELF not found: $elfPath"
}

New-Item -ItemType Directory -Force -Path (Split-Path -Parent $outPath) | Out-Null

$elf = [IO.File]::ReadAllBytes($elfPath)
$e_phoff = [BitConverter]::ToUInt64($elf, 32)
$e_phentsize = [BitConverter]::ToUInt16($elf, 54)
$e_phnum = [BitConverter]::ToUInt16($elf, 56)

# Find max end address
$max = 0
for ($i=0; $i -lt $e_phnum; $i++) {
    $phoff = $e_phoff + $i * $e_phentsize
    $ptype = [BitConverter]::ToUInt32($elf, $phoff)
    if ($ptype -eq 1) {
        $vaddr = [BitConverter]::ToUInt64($elf, $phoff+16)
        $filesz = [BitConverter]::ToUInt64($elf, $phoff+32)
        $end = $vaddr + $filesz
        if ($end -gt $max) { $max = $end }
    }
}

$base = 0x10000
$size = $max - $base
$bin = New-Object byte[] $size
# zero-initialized by default

# Copy each segment into place
for ($i=0; $i -lt $e_phnum; $i++) {
    $phoff = $e_phoff + $i * $e_phentsize
    $ptype = [BitConverter]::ToUInt32($elf, $phoff)
    if ($ptype -eq 1) {
        $offset = [BitConverter]::ToUInt64($elf, $phoff+8)
        $vaddr = [BitConverter]::ToUInt64($elf, $phoff+16)
        $filesz = [BitConverter]::ToUInt64($elf, $phoff+32)
        $src = $elf[$offset..($offset+$filesz-1)]
        $destIndex = $vaddr - $base
        [Array]::Copy($src, 0, $bin, $destIndex, $filesz)
    }
}

[IO.File]::WriteAllBytes($outPath, $bin)
Write-Host "Flat kernel: $($bin.Length) bytes (range 0x$($base.ToString('X'))-0x$($max.ToString('X'))"
