# Build and launch the canonical x86-64 XPARQ OS image in a visible QEMU window.
[CmdletBinding()]
param(
    [switch]$NoBuild,
    [switch]$Gdb,
    [ValidateRange(64, 4096)]
    [int]$MemoryMB = 128
)

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

$repoRoot = $PSScriptRoot
$buildScript = Join-Path $repoRoot "tools\build-x86_64.ps1"
$diskImage = Join-Path $repoRoot "build\x86-64\disk.img"
$serialLog = Join-Path $repoRoot "build\x86-64\gui-serial.log"
$qemuStdout = Join-Path $repoRoot "build\x86-64\qemu-gui.stdout.log"
$qemuStderr = Join-Path $repoRoot "build\x86-64\qemu-gui.stderr.log"

function Resolve-Qemu {
    $command = Get-Command "qemu-system-x86_64" -ErrorAction SilentlyContinue
    if ($command) { return $command.Source }

    foreach ($candidate in @(
        "C:\Program Files\qemu\qemu-system-x86_64.exe",
        "C:\Program Files (x86)\qemu\qemu-system-x86_64.exe"
    )) {
        if (Test-Path -LiteralPath $candidate) {
            return (Resolve-Path -LiteralPath $candidate).Path
        }
    }

    throw "qemu-system-x86_64 was not found. Install QEMU or add it to PATH."
}

function Get-FreeTcpPort {
    $listener = [Net.Sockets.TcpListener]::new([Net.IPAddress]::Loopback, 0)
    try {
        $listener.Start()
        return ([Net.IPEndPoint]$listener.LocalEndpoint).Port
    } finally {
        $listener.Stop()
    }
}

function Invoke-QmpCommand {
    param(
        [Parameter(Mandatory = $true)][IO.StreamWriter]$Writer,
        [Parameter(Mandatory = $true)][IO.StreamReader]$Reader,
        [Parameter(Mandatory = $true)][string]$Json
    )
    $Writer.WriteLine($Json)
    while ($true) {
        $line = $Reader.ReadLine()
        if ($null -eq $line) { throw "QMP connection closed before a response." }
        $response = $line | ConvertFrom-Json
        if ($response.PSObject.Properties.Name -contains "error") {
            throw "QMP command failed: $line"
        }
        if ($response.PSObject.Properties.Name -contains "return") {
            return $response.return
        }
    }
}

Set-Location $repoRoot

if (-not $NoBuild) {
    Write-Host "[INFO] Building and validating the canonical x86-64 image..." -ForegroundColor Cyan
    & $buildScript --scenario gate0 --no-test
    if ($LASTEXITCODE -ne 0) {
        throw "Canonical build failed with exit code $LASTEXITCODE. QEMU was not started."
    }
}

if (-not (Test-Path -LiteralPath $diskImage)) {
    throw "Disk image not found: $diskImage. Run .\start.ps1 without -NoBuild first."
}

$qemu = Resolve-Qemu
$qmpPort = Get-FreeTcpPort
foreach ($logPath in @($serialLog, $qemuStdout, $qemuStderr)) {
    if (Test-Path -LiteralPath $logPath) {
        Remove-Item -LiteralPath $logPath -Force
    }
}

$qemuArgs = @(
    "-machine", "pc",
    "-cpu", "qemu64",
    "-m", "${MemoryMB}M",
    "-drive", "format=raw,file=$diskImage,index=0,media=disk",
    "-boot", "order=c",
    # Keep the guest framebuffer at native resolution so 8x16 glyphs are not
    # resampled into the tiny remembered QEMU window size.
    "-display", "gtk,zoom-to-fit=off",
    "-vga", "std",
    "-monitor", "none",
    "-S",
    "-qmp", "tcp:127.0.0.1:$qmpPort,server=on,wait=off",
    "-serial", "file:$serialLog",
    "-no-reboot",
    "-no-shutdown"
)

if ($Gdb) {
    $qemuArgs += @("-s")
    Write-Host "[INFO] Debug mode: CPU paused; GDB server listening on tcp:1234." -ForegroundColor Yellow
}

Write-Host "[INFO] Starting XPARQ OS in a QEMU window..." -ForegroundColor Cyan
$process = Start-Process -FilePath $qemu -ArgumentList $qemuArgs -PassThru -WindowStyle Normal `
    -RedirectStandardOutput $qemuStdout -RedirectStandardError $qemuStderr

$qmpClient = $null
try {
    $deadline = [DateTime]::UtcNow.AddSeconds(10)
    while (-not $qmpClient -and [DateTime]::UtcNow -lt $deadline) {
        $process.Refresh()
        if ($process.HasExited) {
            $details = (Get-Content -LiteralPath $qemuStderr -Raw -ErrorAction SilentlyContinue).Trim()
            throw "QEMU exited immediately with code $($process.ExitCode). $details"
        }
        try {
            $qmpClient = [Net.Sockets.TcpClient]::new("127.0.0.1", $qmpPort)
        } catch {
            Start-Sleep -Milliseconds 25
        }
    }
    if (-not $qmpClient) { throw "Timed out connecting to QMP on port $qmpPort." }

    $qmpClient.ReceiveTimeout = 2000
    $qmpReader = [IO.StreamReader]::new($qmpClient.GetStream())
    $qmpWriter = [IO.StreamWriter]::new($qmpClient.GetStream())
    $qmpWriter.AutoFlush = $true
    if ($null -eq $qmpReader.ReadLine()) { throw "QMP did not send a greeting." }
    [void](Invoke-QmpCommand $qmpWriter $qmpReader '{"execute":"qmp_capabilities"}')

    if (-not $Gdb) {
        [void](Invoke-QmpCommand $qmpWriter $qmpReader '{"execute":"cont"}')
    }
    $status = Invoke-QmpCommand $qmpWriter $qmpReader '{"execute":"query-status"}'
    $expectedStatus = if ($Gdb) { "paused" } else { "running" }
    if ($status.status -ne $expectedStatus) {
        throw "Unexpected QEMU state '$($status.status)'; expected '$expectedStatus'."
    }
} catch {
    $process.Refresh()
    if (-not $process.HasExited) {
        Stop-Process -Id $process.Id -Force -ErrorAction SilentlyContinue
        $process.WaitForExit()
    }
    throw
} finally {
    if ($qmpClient) { $qmpClient.Dispose() }
}

Write-Host "[SUCCESS] QEMU started (PID $($process.Id))." -ForegroundColor Green
Write-Host "[SUCCESS] QEMU state verified: $expectedStatus." -ForegroundColor Green
Write-Host "[INFO] Serial log: $serialLog"
Write-Host "[INFO] Close the QEMU window to stop this instance."
