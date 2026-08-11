# XPARQ OS canonical x86-64 build and boot smoke-test runner.
# Exit codes: 0 success, 1 dependency/build/layout failure,
#             2 boot timeout, 3 boot failure or early QEMU exit.

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

$PROJECT_ROOT = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$BUILD_DIR = Join-Path $PROJECT_ROOT "build\x86-64"
$TARGET = "x86_64-unknown-none"
$CONFIGURATION = "release"
$CLEAN = $false
$VERBOSE = $false
$RUN_TESTS = $true
$TIMEOUT_SECONDS = 30
$REPEAT = 1
$SCENARIO = "gate0"

$KERNEL_LOAD_SECTORS = 960
$SECTOR_SIZE = 512
$KERNEL_MAX_BYTES = $KERNEL_LOAD_SECTORS * $SECTOR_SIZE
$FAT32_START_LBA = 2048
$SUCCESS_MARKER = "XPARQ_TEST:INIT_READY"
$REQUIRED_MARKERS = @($SUCCESS_MARKER)
$FAILURE_MARKERS = @(
    "XPARQ_TEST:FAIL:",
    "XPARQ_TEST:GATE1_FAIL:",
    "[XPARQ OS] Failed to load init.elf!",
    "[XPARQ OS] HAL init failed!",
    "[XPARQ OS] ERROR: user_rip or user_rsp is zero!"
)

function Write-Info([string]$Message) { Write-Host "[INFO] $Message" -ForegroundColor Cyan }
function Write-Success([string]$Message) { Write-Host "[SUCCESS] $Message" -ForegroundColor Green }
function Write-Warn([string]$Message) { Write-Host "[WARNING] $Message" -ForegroundColor Yellow }

function Fail([string]$Message) {
    throw $Message
}

function Invoke-Checked {
    param(
        [Parameter(Mandatory = $true)][string]$FilePath,
        [Parameter(Mandatory = $true)][string[]]$Arguments,
        [Parameter(Mandatory = $true)][string]$Description
    )

    if ($VERBOSE) {
        Write-Info "$Description`: $FilePath $($Arguments -join ' ')"
    } else {
        Write-Info $Description
    }

    & $FilePath @Arguments
    if ($LASTEXITCODE -ne 0) {
        Fail "$Description failed with exit code $LASTEXITCODE."
    }
}

function Resolve-CommandPath([string]$Name, [string[]]$Fallbacks) {
    $command = Get-Command $Name -ErrorAction SilentlyContinue
    if ($command) { return $command.Source }
    foreach ($candidate in $Fallbacks) {
        if ($candidate -and (Test-Path -LiteralPath $candidate)) {
            return (Resolve-Path -LiteralPath $candidate).Path
        }
    }
    return $null
}

function Resolve-LlvmObjcopy {
    $rustc = & rustup which rustc
    if ($LASTEXITCODE -ne 0 -or -not $rustc) { return $null }
    $rustcDir = Split-Path $rustc.Trim()
    $direct = Join-Path $rustcDir "llvm-objcopy.exe"
    if (Test-Path -LiteralPath $direct) { return $direct }
    $toolchainRoot = Split-Path $rustcDir -Parent
    $found = Get-ChildItem -Path $toolchainRoot -Recurse -Filter "llvm-objcopy.exe" -ErrorAction SilentlyContinue | Select-Object -First 1
    if ($found) { return $found.FullName }
    return $null
}

function Get-ToolVersion([string]$FilePath, [string[]]$Arguments) {
    try {
        $value = & $FilePath @Arguments 2>&1 | Select-Object -First 1
        return "$value".Trim()
    } catch {
        return "unknown"
    }
}

function Parse-Arguments {
    for ($i = 0; $i -lt $args.Count; $i++) {
        switch ($args[$i]) {
            "-t" {
                if ($i + 1 -ge $args.Count) { Fail "-t requires debug or release." }
                $script:CONFIGURATION = $args[++$i]
            }
            "-c" { $script:CLEAN = $true }
            "-v" { $script:VERBOSE = $true }
            "--no-test" { $script:RUN_TESTS = $false }
            "--timeout-seconds" {
                if ($i + 1 -ge $args.Count) { Fail "--timeout-seconds requires a positive integer." }
                $script:TIMEOUT_SECONDS = [int]$args[++$i]
            }
            "--repeat" {
                if ($i + 1 -ge $args.Count) { Fail "--repeat requires a positive integer." }
                $script:REPEAT = [int]$args[++$i]
            }
            "--scenario" {
                if ($i + 1 -ge $args.Count) { Fail "--scenario requires gate0, gate1, gate1-input, gate1-gui, or gate1-fault." }
                $script:SCENARIO = $args[++$i]
            }
            default { Fail "Unknown option: $($args[$i])" }
        }
    }

    if ($CONFIGURATION -notin @("debug", "release")) { Fail "Invalid configuration: $CONFIGURATION" }
    if ($TIMEOUT_SECONDS -lt 1) { Fail "--timeout-seconds must be at least 1." }
    if ($REPEAT -lt 1) { Fail "--repeat must be at least 1." }
    if ($SCENARIO -notin @("gate0", "gate1", "gate1-input", "gate1-gui", "gate1-fault")) { Fail "Invalid scenario: $SCENARIO" }
}

function Configure-Scenario {
    if ($SCENARIO -eq "gate1") {
        $script:SUCCESS_MARKER = "XPARQ_TEST:GATE1_PASS"
        $script:REQUIRED_MARKERS = @(
            "XPARQ_TEST:GATE1:WRITE_OK",
            "XPARQ_TEST:GATE1:SLEEP_OK",
            "XPARQ_TEST:GATE1:ERRORS_OK",
            "XPARQ_TEST:GATE1:FILE_OK",
            "XPARQ_TEST:GATE1:IPC_OK",
            "XPARQ_TEST:GATE1_PASS",
            "XPARQ_TEST:GATE1:EXIT_ENTERED"
        )
    } elseif ($SCENARIO -eq "gate1-input") {
        $script:SUCCESS_MARKER = "XPARQ_TEST:INIT_READY"
        $script:REQUIRED_MARKERS = @(
            "XPARQ_TEST:GATE1:INPUT_INJECTION_READY",
            "XPARQ_TEST:GATE1:KEYBOARD_INPUT_OK",
            "XPARQ_TEST:GATE1:MOUSE_INPUT_OK",
            "XPARQ_TEST:INIT_READY"
        )
    } elseif ($SCENARIO -eq "gate1-gui") {
        $script:SUCCESS_MARKER = "XPARQ_TEST:GUI_TERMINAL_REDRAW_OK"
        $script:REQUIRED_MARKERS = @(
            "XPARQ_DISPLAY:FRAMEBUFFER",
            "XPARQ_GUI:FIRST_FRAME_READY",
            "XPARQ_GUI:RUNNING",
            "XPARQ_TEST:INIT_READY",
            "XPARQ_TEST:GUI_MOUSE_OK",
            "XPARQ_TEST:GUI_DRAG_OK",
            "XPARQ_TEST:GUI_KEYBOARD_OK",
            "XPARQ_TEST:GUI_TERMINAL_REDRAW_OK"
        )
    } elseif ($SCENARIO -eq "gate1-fault") {
        $script:SUCCESS_MARKER = "XPARQ_TEST:FAULT:PAGE_FAULT"
        $script:REQUIRED_MARKERS = @(
            "XPARQ_TEST:FAULT:ARMED",
            "XPARQ_TEST:FAULT:PAGE_FAULT",
            "XPARQ_FAULT:CR2=",
            "XPARQ_FAULT:ERROR="
        )
    }
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

function Invoke-QmpCommand([IO.StreamWriter]$Writer, [IO.StreamReader]$Reader, [string]$Json) {
    $Writer.WriteLine($Json)
    while ($true) {
        $line = $Reader.ReadLine()
        if ($null -eq $line) { Fail "QMP connection closed before a command response." }
        $response = $line | ConvertFrom-Json
        if ($response.PSObject.Properties.Name -contains "error") { Fail "QMP command failed: $line" }
        if ($response.PSObject.Properties.Name -contains "return") { return $response.return }
    }
}

function Save-QmpScreenshot([IO.StreamWriter]$Writer, [IO.StreamReader]$Reader, [string]$Path) {
    $jsonPath = $Path.Replace([IO.Path]::DirectorySeparatorChar, [IO.Path]::AltDirectorySeparatorChar)
    [void](Invoke-QmpCommand $Writer $Reader ("{`"execute`":`"screendump`",`"arguments`":{`"filename`":`"$jsonPath`",`"format`":`"png`"}}"))
    if (-not (Test-Path -LiteralPath $Path)) { Fail "QMP did not create screenshot: $Path" }
}

function Send-QmpKey([IO.StreamWriter]$Writer, [IO.StreamReader]$Reader, [string]$Key) {
    $command = [ordered]@{
        execute = "input-send-event"
        arguments = [ordered]@{
            events = @(
                [ordered]@{ type = "key"; data = [ordered]@{ down = $true; key = [ordered]@{ type = "qcode"; data = $Key } } },
                [ordered]@{ type = "key"; data = [ordered]@{ down = $false; key = [ordered]@{ type = "qcode"; data = $Key } } }
            )
        }
    } | ConvertTo-Json -Compress -Depth 8
    [void](Invoke-QmpCommand $Writer $Reader $command)
}

function Read-BootLogSnapshot([string]$Path) {
    if (-not (Test-Path -LiteralPath $Path)) { return "" }
    $stream = [IO.File]::Open($Path, [IO.FileMode]::Open, [IO.FileAccess]::Read, [IO.FileShare]::ReadWrite)
    try {
        $length = [int][Math]::Min($stream.Length, 1MB)
        if ($length -eq 0) { return "" }
        $buffer = New-Object byte[] $length
        $read = $stream.Read($buffer, 0, $length)
        return [Text.Encoding]::ASCII.GetString($buffer, 0, $read)
    } finally {
        $stream.Dispose()
    }
}

function Resolve-Dependencies {
    $script:Cargo = Resolve-CommandPath "cargo" @()
    $script:Rustc = Resolve-CommandPath "rustc" @()
    $script:Rustup = Resolve-CommandPath "rustup" @()
    $script:Git = Resolve-CommandPath "git" @()
    $script:Nasm = Resolve-CommandPath "nasm" @(
        (Join-Path $PROJECT_ROOT "third_party\nasm\nasm-2.16.03\nasm.exe")
    )
    $script:LlvmObjcopy = Resolve-LlvmObjcopy
    $script:Qemu = Resolve-CommandPath "qemu-system-x86_64" @(
        "C:\Program Files\qemu\qemu-system-x86_64.exe",
        "C:\Program Files (x86)\qemu\qemu-system-x86_64.exe"
    )

    foreach ($dependency in @(
        @{ Name = "cargo"; Path = $Cargo },
        @{ Name = "rustc"; Path = $Rustc },
        @{ Name = "rustup"; Path = $Rustup },
        @{ Name = "git"; Path = $Git },
        @{ Name = "NASM"; Path = $Nasm },
        @{ Name = "llvm-objcopy"; Path = $LlvmObjcopy }
    )) {
        if (-not $dependency.Path) { Fail "$($dependency.Name) was not found." }
    }
    if ($RUN_TESTS -and -not $Qemu) {
        Fail "qemu-system-x86_64 was not found. Install QEMU or use --no-test for build-only validation."
    }

    $installedTargets = & $Rustup target list --installed
    if ($LASTEXITCODE -ne 0 -or $installedTargets -notcontains $TARGET) {
        Fail "Rust target $TARGET is not installed. Run: rustup target add $TARGET"
    }
}

function Get-CargoBuildArguments([string]$Package) {
    $result = @("build", "--target", $TARGET, "--package", $Package)
    if ($CONFIGURATION -eq "release") { $result += "--release" }
    if (-not $VERBOSE) { $result += "--quiet" }
    return $result
}

function Invoke-TargetedSafetyChecks {
    Write-Info "Running targeted compile checks with static_mut_refs denied."
    $previousRustFlags = $env:RUSTFLAGS
    try {
        $env:RUSTFLAGS = "-Dstatic_mut_refs"
        foreach ($package in @("xparq-kernel", "xparq-hal", "init")) {
            $checkArgs = @("check", "--target", $TARGET, "--package", $package)
            if ($package -eq "xparq-kernel" -and $SCENARIO -eq "gate1-input") {
                $checkArgs += @("--features", "gate1-test")
            }
            if ($package -eq "init" -and $SCENARIO -in @("gate1", "gate1-input", "gate1-fault")) {
                $feature = if ($SCENARIO -eq "gate1") { "gate1-test" } elseif ($SCENARIO -eq "gate1-input") { "gate1-input-test" } else { "gate1-fault-test" }
                $checkArgs += @("--features", $feature)
            }
            if (-not $VERBOSE) { $checkArgs += "--quiet" }
            Invoke-Checked $Cargo $checkArgs "Checking $package"
        }
    } finally {
        $env:RUSTFLAGS = $previousRustFlags
    }
}

function Invoke-Gate1HostTests {
    if ($SCENARIO -notin @("gate1", "gate1-fault")) { return }
    $testArgs = @("test", "--package", "xparq-hal", "--lib")
    if (-not $VERBOSE) { $testArgs += "--quiet" }
    Invoke-Checked $Cargo $testArgs "Running Gate 1 MBR/FAT32 host parser tests"
}

function Build-Inputs {
    New-Item -ItemType Directory -Force -Path $BUILD_DIR | Out-Null

    $kernelArgs = Get-CargoBuildArguments "xparq-kernel"
    if ($SCENARIO -eq "gate1-input") { $kernelArgs += @("--features", "gate1-test") }
    if ($SCENARIO -eq "gate1-gui") { $kernelArgs += @("--features", "gate1-gui-test") }
    Invoke-Checked $Cargo $kernelArgs "Building kernel ELF"
    $initArgs = Get-CargoBuildArguments "init"
    if ($SCENARIO -eq "gate1") { $initArgs += @("--features", "gate1-test") }
    if ($SCENARIO -eq "gate1-input") { $initArgs += @("--features", "gate1-input-test") }
    if ($SCENARIO -eq "gate1-fault") { $initArgs += @("--features", "gate1-fault-test") }
    Invoke-Checked $Cargo $initArgs "Building user-space init for $SCENARIO"

    $profileDir = if ($CONFIGURATION -eq "release") { "release" } else { "debug" }
    $kernelElf = Join-Path $PROJECT_ROOT "target\$TARGET\$profileDir\xparq_kernel"
    $initElf = Join-Path $PROJECT_ROOT "target\$TARGET\$profileDir\init"
    if (-not (Test-Path -LiteralPath $kernelElf)) { Fail "Kernel ELF not found: $kernelElf" }
    if (-not (Test-Path -LiteralPath $initElf)) { Fail "Init ELF not found: $initElf" }

    $kernelBin = Join-Path $BUILD_DIR "kernel.bin"
    Invoke-Checked $LlvmObjcopy @(
        "-O", "binary", "-R", ".bss", "-R", ".comment", "-R", ".symtab",
        "-R", ".shstrtab", "-R", ".strtab", $kernelElf, $kernelBin
    ) "Converting kernel ELF to raw binary"

    $kernelLength = (Get-Item -LiteralPath $kernelBin).Length
    if ($kernelLength -gt $KERNEL_MAX_BYTES) {
        Fail "Kernel is $kernelLength bytes; bootloader load window is $KERNEL_MAX_BYTES bytes."
    }

    $bootloaderBin = Join-Path $BUILD_DIR "bootloader.bin"
    $bootloaderAsm = Join-Path $PROJECT_ROOT "bootloader\x86_64\src\boot.asm"
    Invoke-Checked $Nasm @("-f", "bin", $bootloaderAsm, "-o", $bootloaderBin) "Assembling bootloader"

    Invoke-Checked $Cargo @("build", "--release", "--package", "fat32-injector") "Building FAT32 injector"
    $injector = Join-Path $PROJECT_ROOT "target\release\fat32-injector.exe"
    if (-not (Test-Path -LiteralPath $injector)) {
        $injector = Join-Path $PROJECT_ROOT "target\release\fat32-injector"
    }
    if (-not (Test-Path -LiteralPath $injector)) { Fail "FAT32 injector executable was not found." }

    $fat32Image = Join-Path $BUILD_DIR "fat32.img"
    if (Test-Path -LiteralPath $fat32Image) { Remove-Item -LiteralPath $fat32Image -Force }
    Invoke-Checked $injector @($fat32Image, $initElf, "INIT.ELF") "Creating FAT32 image and injecting INIT.ELF"
    if ($SCENARIO -in @("gate1", "gate1-fault")) {
        $fixturePath = Join-Path $BUILD_DIR "gate1.txt"
        [IO.File]::WriteAllBytes($fixturePath, [Text.Encoding]::ASCII.GetBytes("XPARQ_GATE1_FILE_OK`n"))
        Invoke-Checked $injector @($fat32Image, $fixturePath, "GATE1.TXT") "Injecting Gate 1 file fixture"
    }
}

function Build-AndValidateDiskImage {
    $bootloaderPath = Join-Path $BUILD_DIR "bootloader.bin"
    $kernelPath = Join-Path $BUILD_DIR "kernel.bin"
    $fat32Path = Join-Path $BUILD_DIR "fat32.img"
    $diskPath = Join-Path $BUILD_DIR "disk.img"

    $bootloader = [IO.File]::ReadAllBytes($bootloaderPath)
    $kernel = [IO.File]::ReadAllBytes($kernelPath)
    $fat32 = [IO.File]::ReadAllBytes($fat32Path)

    if ($bootloader.Length -ne $SECTOR_SIZE) { Fail "Bootloader must be exactly 512 bytes; got $($bootloader.Length)." }
    if ($bootloader[510] -ne 0x55 -or $bootloader[511] -ne 0xAA) { Fail "Bootloader signature 55 AA is missing." }
    if ($kernel.Length -gt $KERNEL_MAX_BYTES) { Fail "Kernel exceeds the 960-sector load window." }
    if (($fat32.Length % $SECTOR_SIZE) -ne 0) { Fail "FAT32 image size is not sector-aligned." }

    for ($offset = 446; $offset -lt 510; $offset++) {
        if ($bootloader[$offset] -ne 0) { Fail "Bootloader overlaps the MBR partition-table region at byte $offset." }
    }

    $partitionSectors = [uint32]($fat32.Length / $SECTOR_SIZE)
    $bootloader[446] = 0x00
    $bootloader[450] = 0x0C
    [Array]::Copy([BitConverter]::GetBytes([uint32]$FAT32_START_LBA), 0, $bootloader, 454, 4)
    [Array]::Copy([BitConverter]::GetBytes($partitionSectors), 0, $bootloader, 458, 4)
    $bootloader[510] = 0x55
    $bootloader[511] = 0xAA

    $fatOffset = $FAT32_START_LBA * $SECTOR_SIZE
    if (($SECTOR_SIZE + $KERNEL_MAX_BYTES) -gt $fatOffset) { Fail "Kernel load window overlaps FAT32 partition." }
    $disk = New-Object byte[] ($fatOffset + $fat32.Length)
    [Array]::Copy($bootloader, 0, $disk, 0, $bootloader.Length)
    [Array]::Copy($kernel, 0, $disk, $SECTOR_SIZE, $kernel.Length)
    [Array]::Copy($fat32, 0, $disk, $fatOffset, $fat32.Length)
    [IO.File]::WriteAllBytes($diskPath, $disk)

    $written = [IO.File]::ReadAllBytes($diskPath)
    $startLba = [BitConverter]::ToUInt32($written, 454)
    $sectorCount = [BitConverter]::ToUInt32($written, 458)
    if ($written[510] -ne 0x55 -or $written[511] -ne 0xAA) { Fail "Written disk image has an invalid boot signature." }
    if ($startLba -ne $FAT32_START_LBA -or $sectorCount -ne $partitionSectors) { Fail "Written partition table does not match the FAT32 layout." }
    if ($written.Length -ne ($fatOffset + $fat32.Length)) { Fail "Written disk image has an unexpected length." }

    Write-Success "Validated disk image: $diskPath"
}

function Write-BuildManifest([bool]$RuntimeVerified, [int]$PassedRuns, [int]$RequestedRuns) {
    $artifacts = [ordered]@{}
    foreach ($name in @("bootloader.bin", "kernel.bin", "fat32.img", "disk.img")) {
        $path = Join-Path $BUILD_DIR $name
        $item = Get-Item -LiteralPath $path
        $artifacts[$name] = [ordered]@{
            size = $item.Length
            sha256 = (Get-FileHash -Algorithm SHA256 -LiteralPath $path).Hash.ToLowerInvariant()
        }
    }

    $evidence = [ordered]@{}
    if ($SCENARIO -eq "gate1-gui") {
        foreach ($path in Get-ChildItem -LiteralPath $BUILD_DIR -Filter "gate1-gui.*.run-*.png" -File -ErrorAction SilentlyContinue) {
            $evidence[$path.Name] = [ordered]@{
                size = $path.Length
                sha256 = (Get-FileHash -Algorithm SHA256 -LiteralPath $path.FullName).Hash.ToLowerInvariant()
            }
        }
    }

    $commit = (& $Git -C $PROJECT_ROOT rev-parse HEAD).Trim()
    if ($LASTEXITCODE -ne 0) { Fail "Unable to resolve Git commit for build manifest." }
    $manifest = [ordered]@{
        schemaVersion = 1
        generatedAtUtc = [DateTime]::UtcNow.ToString("o")
        deterministicFields = @("commit", "configuration", "toolVersions", "artifacts")
        nonDeterministicFields = @("generatedAtUtc", "runtime")
        commit = $commit
        configuration = $CONFIGURATION
        scenario = $SCENARIO
        target = $TARGET
        imageLayout = [ordered]@{
            bootLba = 0
            kernelStartLba = 1
            kernelLoadSectors = $KERNEL_LOAD_SECTORS
            fat32StartLba = $FAT32_START_LBA
        }
        toolVersions = [ordered]@{
            rustc = Get-ToolVersion $Rustc @("--version")
            cargo = Get-ToolVersion $Cargo @("--version")
            nasm = Get-ToolVersion $Nasm @("-v")
            llvmObjcopy = Get-ToolVersion $LlvmObjcopy @("--version")
            qemu = if ($Qemu) { Get-ToolVersion $Qemu @("--version") } else { "not-run" }
        }
        artifacts = $artifacts
        evidence = $evidence
        runtime = [ordered]@{
            verified = $RuntimeVerified
            successMarker = $SUCCESS_MARKER
            requiredMarkers = $REQUIRED_MARKERS
            passedRuns = $PassedRuns
            requestedRuns = $RequestedRuns
        }
    }
    $manifestJson = $manifest | ConvertTo-Json -Depth 8
    $manifestJson | Set-Content -LiteralPath (Join-Path $BUILD_DIR "build-manifest.json") -Encoding UTF8
    $manifestJson | Set-Content -LiteralPath (Join-Path $BUILD_DIR ("build-manifest.{0}.json" -f $SCENARIO)) -Encoding UTF8
}

function Save-ScenarioBootLog([string]$AggregateLog) {
    if (Test-Path -LiteralPath $AggregateLog) {
        Copy-Item -LiteralPath $AggregateLog -Destination (Join-Path $BUILD_DIR ("boot.{0}.log" -f $SCENARIO)) -Force
    }
}

function Invoke-BootRun([int]$RunNumber) {
    $runLog = Join-Path $BUILD_DIR ("boot.run-{0:D2}.log" -f $RunNumber)
    $stdoutLog = Join-Path $BUILD_DIR ("qemu.run-{0:D2}.stdout.log" -f $RunNumber)
    $stderrLog = Join-Path $BUILD_DIR ("qemu.run-{0:D2}.stderr.log" -f $RunNumber)
    $diskPath = Join-Path $BUILD_DIR "disk.img"
    $usesQmp = $SCENARIO -in @("gate1-input", "gate1-gui")
    $qmpPort = if ($usesQmp) { Get-FreeTcpPort } else { 0 }
    $beforeScreenshot = Join-Path $BUILD_DIR ("gate1-gui.before.run-{0:D2}.png" -f $RunNumber)
    $afterScreenshot = Join-Path $BUILD_DIR ("gate1-gui.after.run-{0:D2}.png" -f $RunNumber)
    foreach ($path in @($runLog, $stdoutLog, $stderrLog, $beforeScreenshot, $afterScreenshot)) {
        if (Test-Path -LiteralPath $path) { Remove-Item -LiteralPath $path -Force }
    }
    $qemuArgs = @(
        "-machine", "pc",
        "-cpu", "qemu64",
        "-m", "128M",
        "-drive", "format=raw,file=$diskPath,index=0,media=disk",
        "-boot", "order=c",
        "-display", "none",
        "-monitor", "none",
        "-serial", "file:$runLog",
        "-no-reboot"
    )
    if ($usesQmp) {
        # Start paused so the runner owns QMP before the very short boot begins.
        $qemuArgs += @(
            "-S",
            "-qmp", "tcp:127.0.0.1:$qmpPort,server=on,wait=off"
        )
    }

    $process = Start-Process -FilePath $Qemu -ArgumentList $qemuArgs -PassThru -WindowStyle Hidden `
        -RedirectStandardOutput $stdoutLog -RedirectStandardError $stderrLog
    $deadline = [DateTime]::UtcNow.AddSeconds($TIMEOUT_SECONDS)
    $qmpClient = $null
    $qmpReader = $null
    $qmpWriter = $null
    $inputInjected = $false
    try {
        if ($usesQmp) {
            while ([DateTime]::UtcNow -lt $deadline -and -not $qmpClient) {
                $process.Refresh()
                if ($process.HasExited) { return 3 }
                try {
                    $qmpClient = [Net.Sockets.TcpClient]::new("127.0.0.1", $qmpPort)
                } catch {
                    Start-Sleep -Milliseconds 20
                }
            }
            if (-not $qmpClient) { return 3 }
            $qmpClient.ReceiveTimeout = 2000
            $qmpReader = [IO.StreamReader]::new($qmpClient.GetStream())
            $qmpWriter = [IO.StreamWriter]::new($qmpClient.GetStream())
            $qmpWriter.AutoFlush = $true
            if ($null -eq $qmpReader.ReadLine()) { return 3 }
            [void](Invoke-QmpCommand $qmpWriter $qmpReader '{"execute":"qmp_capabilities"}')
            [void](Invoke-QmpCommand $qmpWriter $qmpReader '{"execute":"cont"}')
            $status = Invoke-QmpCommand $qmpWriter $qmpReader '{"execute":"query-status"}'
            if ($status.status -ne "running") { return 3 }
        }

        while ([DateTime]::UtcNow -lt $deadline) {
            $process.Refresh()
            $content = Read-BootLogSnapshot $runLog
            foreach ($marker in $FAILURE_MARKERS) {
                if ($content -and $content.Contains($marker)) { return 3 }
            }
            if ($SCENARIO -eq "gate1-input" -and -not $inputInjected -and
                $content -and $content.Contains("XPARQ_TEST:GATE1:INPUT_INJECTION_READY")) {
                for ($batch = 0; $batch -lt 4; $batch++) {
                    [void](Invoke-QmpCommand $qmpWriter $qmpReader '{"execute":"input-send-event","arguments":{"events":[{"type":"key","data":{"down":true,"key":{"type":"qcode","data":"a"}}},{"type":"key","data":{"down":false,"key":{"type":"qcode","data":"a"}}}]}}')
                    [void](Invoke-QmpCommand $qmpWriter $qmpReader '{"execute":"input-send-event","arguments":{"events":[{"type":"rel","data":{"axis":"x","value":3}},{"type":"rel","data":{"axis":"y","value":-2}},{"type":"btn","data":{"down":true,"button":"left"}},{"type":"btn","data":{"down":false,"button":"left"}}]}}')
                }
                $inputInjected = $true
            }
            if ($SCENARIO -eq "gate1-gui" -and -not $inputInjected -and $content -and
                $content.Contains("XPARQ_GUI:RUNNING") -and $content.Contains("XPARQ_TEST:INIT_READY")) {
                Save-QmpScreenshot $qmpWriter $qmpReader $beforeScreenshot
                # Cursor starts at 512,384. Move onto the terminal title bar,
                # press, drag the window, release, then type `help` in Ring 3.
                for ($step = 0; $step -lt 5; $step++) {
                    [void](Invoke-QmpCommand $qmpWriter $qmpReader '{"execute":"input-send-event","arguments":{"events":[{"type":"rel","data":{"axis":"x","value":-42}},{"type":"rel","data":{"axis":"y","value":-55}}]}}')
                    Start-Sleep -Milliseconds 20
                }
                [void](Invoke-QmpCommand $qmpWriter $qmpReader '{"execute":"input-send-event","arguments":{"events":[{"type":"btn","data":{"down":true,"button":"left"}}]}}')
                Start-Sleep -Milliseconds 20
                for ($step = 0; $step -lt 5; $step++) {
                    [void](Invoke-QmpCommand $qmpWriter $qmpReader '{"execute":"input-send-event","arguments":{"events":[{"type":"rel","data":{"axis":"x","value":10}},{"type":"rel","data":{"axis":"y","value":6}}]}}')
                    Start-Sleep -Milliseconds 20
                }
                [void](Invoke-QmpCommand $qmpWriter $qmpReader '{"execute":"input-send-event","arguments":{"events":[{"type":"btn","data":{"down":false,"button":"left"}}]}}')
                Start-Sleep -Milliseconds 20
                foreach ($key in @("h", "e", "l", "p", "ret")) {
                    Send-QmpKey $qmpWriter $qmpReader $key
                    Start-Sleep -Milliseconds 20
                }
                $inputInjected = $true
            }
            if ($content) {
                $allMarkersFound = $true
                foreach ($marker in $REQUIRED_MARKERS) {
                    if (-not $content.Contains($marker)) { $allMarkersFound = $false; break }
                }
                if ($allMarkersFound) {
                    if ($SCENARIO -eq "gate1-gui") {
                        Save-QmpScreenshot $qmpWriter $qmpReader $afterScreenshot
                        $beforeHash = (Get-FileHash -Algorithm SHA256 -LiteralPath $beforeScreenshot).Hash
                        $afterHash = (Get-FileHash -Algorithm SHA256 -LiteralPath $afterScreenshot).Hash
                        if ($beforeHash -eq $afterHash) { return 3 }
                    }
                    return 0
                }
            }
            if ($process.HasExited) { return 3 }
            Start-Sleep -Milliseconds $(if ($usesQmp) { 5 } else { 100 })
        }
        return 2
    } finally {
        if ($qmpClient) { $qmpClient.Dispose() }
        $process.Refresh()
        if (-not $process.HasExited) { Stop-Process -Id $process.Id -Force -ErrorAction SilentlyContinue }
        $process.WaitForExit()
    }
}

function Invoke-SmokeTests {
    $aggregateLog = Join-Path $BUILD_DIR "boot.log"
    if (Test-Path -LiteralPath $aggregateLog) { Remove-Item -LiteralPath $aggregateLog -Force }
    $passed = 0
    for ($run = 1; $run -le $REPEAT; $run++) {
        Write-Info "Boot smoke test $run/$REPEAT"
        $result = Invoke-BootRun $run
        $runLog = Join-Path $BUILD_DIR ("boot.run-{0:D2}.log" -f $run)
        Add-Content -LiteralPath $aggregateLog -Value "===== RUN $run/$REPEAT (exit $result) ====="
        if (Test-Path -LiteralPath $runLog) { Get-Content -LiteralPath $runLog | Add-Content -LiteralPath $aggregateLog }

        if ($result -eq 0) {
            $passed++
            Write-Success "Boot run $run reached $SUCCESS_MARKER."
        } elseif ($result -eq 2) {
            Save-ScenarioBootLog $aggregateLog
            Write-BuildManifest $false $passed $REPEAT
            Write-Warn "Boot run $run timed out after $TIMEOUT_SECONDS seconds."
            exit 2
        } else {
            Save-ScenarioBootLog $aggregateLog
            Write-BuildManifest $false $passed $REPEAT
            Write-Warn "Boot run $run failed before reaching $SUCCESS_MARKER."
            exit 3
        }
    }
    Save-ScenarioBootLog $aggregateLog
    Write-BuildManifest $true $passed $REPEAT
    Write-Success "Boot stability result: $passed/$REPEAT runs passed."
}

try {
    Parse-Arguments @args
    Configure-Scenario
    Set-Location $PROJECT_ROOT
    Write-Info "XPARQ OS canonical x86-64 $SCENARIO runner"
    Resolve-Dependencies

    if ($CLEAN) {
        Write-Info "Cleaning generated x86-64 artifacts."
        if (Test-Path -LiteralPath $BUILD_DIR) { Remove-Item -LiteralPath $BUILD_DIR -Recurse -Force }
        Invoke-Checked $Cargo @("clean") "Cleaning Cargo artifacts"
    }

    Invoke-TargetedSafetyChecks
    Invoke-Gate1HostTests
    Build-Inputs
    Build-AndValidateDiskImage
    Write-BuildManifest $false 0 $REPEAT

    if ($RUN_TESTS) {
        Invoke-SmokeTests
    } else {
        Write-Warn "Build and layout validation passed; runtime is not verified because --no-test was used."
    }

    Write-Success "$SCENARIO runner completed successfully."
    exit 0
} catch {
    Write-Host "[ERROR] $($_.Exception.Message)" -ForegroundColor Red
    exit 1
}
