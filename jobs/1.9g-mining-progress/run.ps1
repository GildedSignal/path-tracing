param(
    [string]$ServerJar = 'D:/path-tracing-lab/server/versions/26.2/server-26.2.jar',
    [string]$JdkBin = 'D:/path-tracing-deps/data/java/java-runtime-epsilon/bin',
    [string]$Scratch = 'D:/path-tracing-target/1.9g-mining-progress'
)
$ErrorActionPreference = 'Stop'
$PSNativeCommandUseErrorActionPreference = $false
$root = (Resolve-Path "$PSScriptRoot/../..").Path
. "$root/scripts/common.ps1"

function Get-Code([string[]]$Dump, [string]$Method) {
    $code = @{}
    $inside = $false
    foreach ($line in $Dump) {
        if ($line -match '^  \S') {
            if ($inside) { break }
            $inside = $line -match $Method
        }
        if ($inside -and $line -match '^\s+(\d+):\s+(.+)$') {
            $code[[int]$Matches[1]] = $Matches[2] -replace '\s+', ' '
        }
    }
    if ($code.Count -eq 0) { throw "Missing bytecode for $Method" }
    $code
}

function Assert-Code($Code, [string[]]$Rows) {
    foreach ($row in $Rows) {
        $offset, $pattern = $row.Split(' ', 2)
        $instruction = $Code[[int]$offset]
        if ($null -eq $instruction -or $instruction -cnotmatch "^(?:$pattern)$") {
            throw "Bytecode at offset ${offset}: expected $pattern, got $instruction"
        }
    }
}

function Save-Disassembly([string]$Classpath, [string]$Class, [string]$Name) {
    $dump = & "$JdkBin/javap.exe" -cp $Classpath -c -p $Class
    if ($LASTEXITCODE -ne 0) { throw "javap failed for $Class" }
    $dump | Set-Content -LiteralPath "$result/$Name.jbc" -Encoding ASCII
    $dump
}

function Invoke-Comparison([string]$Name, [string]$Test) {
    $exe = "$result/$Name.exe"
    & rustc --edition 2024 --test -O $Test -o $exe
    if ($LASTEXITCODE -ne 0) { throw "$Name failed to build" }
    $log = "$result/$Name.log"
    & $exe --ignored --nocapture --test-threads=1 *> $log
    $code = $LASTEXITCODE
    Get-Content -LiteralPath $log | Write-Host
    $code
}

$stopwatch = [Diagnostics.Stopwatch]::StartNew()
$Rig = (New-Item -ItemType Directory -Force -Path $Scratch).FullName
Enter-Rig '1.9g-mining-progress-offline'
$previousOracle = $env:MINING_ORACLE
try {
    $result = (New-Item -ItemType Directory -Path (
        Join-Path $Rig ([DateTime]::UtcNow.ToString('yyyyMMddTHHmmssfffffffZ'))
    )).FullName
    Write-Host "Evidence $result"

    $hash = (Get-FileHash -LiteralPath $ServerJar -Algorithm SHA256).Hash
    if ($hash -ne '183c0499c5f855570ee487dd38e141a53f0121f83a0b07a3bac2d8b6698823e8') {
        throw "Unexpected 26.2 server jar SHA-256 $hash"
    }
    Write-Host "Vanilla 26.2 SHA-256 $hash"

    $dump = Save-Disassembly $ServerJar 'net.minecraft.server.level.ServerPlayerGameMode' 'game-mode'
    $tick = Get-Code $dump 'public void tick\('
    Assert-Code $tick @(
        '2 getfield .*gameTicks:I', '5 iconst_1', '6 iadd', '7 putfield .*gameTicks:I',
        '11 getfield .*hasDelayedDestroy:Z', '14 ifeq 81',
        '22 getfield .*delayedDestroyPos:.*', '30 invokevirtual .*isAir:.*',
        '33 ifeq 44', '38 putfield .*hasDelayedDestroy:Z', '41 goto 78',
        '51 getfield .*delayedTickStart:I', '54 invokevirtual .*incrementDestroyProgress:.*',
        '58 fload_2', '59 fconst_1', '60 fcmpl', '61 iflt 78',
        '66 putfield .*hasDelayedDestroy:Z', '74 invokevirtual .*destroyBlock:.*',
        '78 goto 153', '82 getfield .*isDestroyingBlock:Z', '85 ifeq 153',
        '93 getfield .*destroyPos:.*', '101 invokevirtual .*isAir:.*', '104 ifeq 139',
        '133 putfield .*isDestroyingBlock:Z', '146 getfield .*destroyProgressStart:I',
        '149 invokevirtual .*incrementDestroyProgress:.*', '152 pop', '153 return'
    )
    $increment = Get-Code $dump 'private float incrementDestroyProgress\('
    Assert-Code $increment @(
        '1 getfield .*gameTicks:I', '4 iload_3', '5 isub', '6 istore 4',
        '21 invokevirtual .*getDestroyProgress:.*', '24 iload 4', '26 iconst_1',
        '27 iadd', '28 i2f', '29 fmul', '72 fload 5', '74 freturn'
    )
    $packet = Get-Code $dump 'public void handleBlockBreakAction\('
    Assert-Code $packet @(
        '254 getfield .*gameTicks:I', '257 putfield .*destroyProgressStart:I',
        '329 invokevirtual .*attack:.*', '346 invokevirtual .*getDestroyProgress:.*',
        '353 invokevirtual .*isAir:.*', '356 ifne 379', '359 fload 6', '361 fconst_1',
        '362 fcmpl', '363 iflt 379', '373 invokevirtual .*destroyAndAck:.*',
        '431 putfield .*isDestroyingBlock:Z', '439 putfield .*destroyPos:.*',
        '488 getstatic .*STOP_DESTROY_BLOCK:.*', '496 getfield .*destroyPos:.*',
        '499 invokevirtual .*equals:.*', '502 ifeq 630', '506 getfield .*gameTicks:I',
        '510 getfield .*destroyProgressStart:I', '513 isub', '514 istore 6',
        '528 invokevirtual .*isAir:.*', '531 ifne 630', '548 invokevirtual .*getDestroyProgress:.*',
        '551 iload 6', '553 iconst_1', '554 iadd', '555 i2f', '556 fmul',
        '559 fload 8', '561 ldc_w .*float 0\.7f', '564 fcmpl', '565 iflt 600',
        '570 putfield .*isDestroyingBlock:Z', '596 invokevirtual .*destroyAndAck:.*',
        '601 getfield .*hasDelayedDestroy:Z', '604 ifne 630',
        '609 putfield .*isDestroyingBlock:Z', '614 putfield .*hasDelayedDestroy:Z',
        '619 putfield .*delayedDestroyPos:.*', '624 getfield .*destroyProgressStart:I',
        '627 putfield .*delayedTickStart:I'
    )

    $block = 'net.minecraft.world.level.block.state.BlockBehaviour'
    $baseDump = Save-Disassembly $ServerJar ($block + '$BlockStateBase') 'block-state'
    Assert-Code (Get-Code $baseDump 'public float getDestroyProgress\(') @(
        '0 aload_0', '11 invokevirtual .*Block.getDestroyProgress:.*', '14 freturn'
    )
    $blockDump = Save-Disassembly $ServerJar $block 'block-behaviour'
    Assert-Code (Get-Code $blockDump 'protected float getDestroyProgress\(') @(
        '4 invokevirtual .*BlockState.getDestroySpeed:.*', '11 ldc_w .*float -1\.0f',
        '14 fcmpl', '15 ifne 20', '18 fconst_0', '19 freturn',
        '22 invokevirtual .*hasCorrectToolForDrops:.*', '25 ifeq 33', '28 bipush 30',
        '30 goto 35', '33 bipush 100', '39 invokevirtual .*Player.getDestroySpeed:.*',
        '42 fload 5', '44 fdiv', '45 iload 6', '47 i2f', '48 fdiv', '49 freturn'
    )

    & "$JdkBin/javac.exe" -d $result "$PSScriptRoot/MiningOracle.java"
    if ($LASTEXITCODE -ne 0) { throw 'Java oracle build failed' }
    $oracleDump = Save-Disassembly $result 'MiningOracle' 'oracle'
    Assert-Code (Get-Code $oracleDump 'float perTick\(float, float, boolean\)') @(
        '0 fload_1', '1 ldc .*float -1\.0f', '3 fcmpl', '4 ifne 9', '7 fconst_0', '8 freturn',
        '9 iload_2', '10 ifeq 18', '13 bipush 30', '15 goto 20', '18 bipush 100', '20 istore_3',
        '21 fload_0', '22 fload_1', '23 fdiv', '24 iload_3', '25 i2f', '26 fdiv', '27 freturn'
    )
    Assert-Code (Get-Code $oracleDump 'float elapsed\(') @(
        '0 iload_1', '1 iload_2', '2 isub', '3 istore_3', '4 fload_0',
        '5 iload_3', '6 iconst_1', '7 iadd', '8 i2f', '9 fmul', '10 freturn'
    )
    Assert-Code (Get-Code $oracleDump 'boolean atOne\(') @(
        '0 fload_0', '1 fconst_1', '2 fcmpl', '3 iflt 10'
    )
    Assert-Code (Get-Code $oracleDump 'boolean atStop\(') @(
        '0 fload_0', '1 ldc .*float 0\.7f', '3 fcmpl', '4 iflt 11'
    )
    Write-Host 'Vanilla sites and compiled JVM arithmetic pinned'

    $env:MINING_ORACLE = "$result/oracle.tsv"
    & "$JdkBin/java.exe" -Xint -cp $result MiningOracle |
        Set-Content -LiteralPath $env:MINING_ORACLE -Encoding ASCII
    if ($LASTEXITCODE -ne 0) { throw 'Java oracle execution failed' }

    $normal = Invoke-Comparison 'original' "$root/engine/law/tests/mining.rs"
    $mutant = "$result/mutant"
    $null = New-Item -ItemType Directory -Path "$mutant/src", "$mutant/tests"
    $source = Get-Content -LiteralPath "$root/engine/law/src/mining.rs" -Raw
    $needle = 'is_delayed && progress >= 1.0'
    if ([regex]::Matches($source, [regex]::Escape($needle)).Count -ne 1) {
        throw 'Expected exactly one delayed completion predicate'
    }
    $source.Replace($needle, 'is_delayed && progress > 1.0') |
        Set-Content -LiteralPath "$mutant/src/mining.rs" -Encoding utf8
    Copy-Item -LiteralPath "$root/engine/law/tests/mining.rs" -Destination "$mutant/tests/mining.rs"
    $control = Invoke-Comparison 'strict-delayed-threshold' "$mutant/tests/mining.rs"
    $first = Select-String -LiteralPath "$result/strict-delayed-threshold.log" -Pattern '^JVM row: '
    Write-Host "Original exit $normal, planted mutant exit $control"
    if ($normal -ne 0 -or $control -ne 101 -or
        $first.Line -cnotmatch '^JVM row: delayed-boundary-0\|tick\|.*\|destroy:4:71:3f800000$') {
        throw 'Mining progress gate failed'
    }
} finally {
    $env:MINING_ORACLE = $previousOracle
    Exit-Rig
    Write-Host ('Gate wall time {0:F3} s' -f $stopwatch.Elapsed.TotalSeconds)
}
