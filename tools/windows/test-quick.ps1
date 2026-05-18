# Quick test
Get-Process qemu-system-x86_64 -ErrorAction SilentlyContinue | Stop-Process -Force
Start-Sleep 1

$env:PATH += ";C:\Program Files\qemu"

# Run and wait
$psi = New-Object System.Diagnostics.ProcessStartInfo
$psi.FileName = "qemu-system-x86_64"
$psi.Arguments = "-drive format=raw,file=build/x86-64/disk.img -nographic -no-reboot -m 128M"
$psi.UseShellExecute = $false
$psi.RedirectStandardOutput = $true
$psi.RedirectStandardError = $true

$proc = [System.Diagnostics.Process]::Start($psi)
Start-Sleep 3

$output = $proc.StandardOutput.ReadToEnd()
$error = $proc.StandardError.ReadToEnd()

if (!$proc.HasExited) { $proc.Kill() }

Write-Host "=== STDOUT ==="
Write-Host $output
Write-Host "=== STDERR ==="
Write-Host $error
