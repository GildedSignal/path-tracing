param([string]$Output = 'D:\path-tracing-evidence\3.2d-3-A-observer')
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot/../../scripts/common.ps1"
$classes = Join-Path $Output 'classes'
New-Item -ItemType Directory -Force $classes | Out-Null
$sources = @('NativeChannel.java','HopCapture.java','mixin/HopConnectionMixin.java','mixin/HopConnectionAccess.java','mixin/HopServerMixin.java','mixin/HopInputMixin.java','mixin/HopRconMixin.java','mixin/FoodStateAccess.java','mixin/LivingStateAccess.java') | ForEach-Object { Join-Path "$Root/bot/src/pathtracing" $_ }
$sources += (Get-ChildItem "$PSScriptRoot/observer" -Filter '*.java').FullName
& "$Jdk/javac.exe" --release 25 -proc:none -encoding UTF-8 -cp ((Get-GameClasspath) -join ';') -d $classes @sources
if ($LASTEXITCODE) { throw 'Observer compilation failed' }
'{"schemaVersion":1,"id":"pathtracing_observer","version":"0.1.0","name":"Path tracing passive baseline observer","environment":"*","mixins":["observer-common.json","observer-client.json"],"depends":{"fabricloader":">=0.19.5","minecraft":"26.2","java":">=25"}}' | Set-Content "$classes/fabric.mod.json" -Encoding ASCII
'{"required":true,"package":"pathtracing.mixin","compatibilityLevel":"JAVA_25","mixins":["HopConnectionMixin","HopConnectionAccess"],"server":["HopServerMixin","HopInputMixin","HopRconMixin","FoodStateAccess","LivingStateAccess"],"injectors":{"defaultRequire":1}}' | Set-Content "$classes/observer-common.json" -Encoding ASCII
'{"required":true,"package":"pathtracing.observer","compatibilityLevel":"JAVA_25","client":["HopBaselinePointerMixin","HopBaselineTrackMixin","HopBaselineBotMixin","HopBaselineRestoreMixin"],"injectors":{"defaultRequire":1}}' | Set-Content "$classes/observer-client.json" -Encoding ASCII
& "$Jdk/jar.exe" --create --file "$Output/observer-only.jar" -C $classes .
if ($LASTEXITCODE) { throw 'Observer packing failed' }
$tests = Join-Path $Output 'tests'
New-Item -ItemType Directory -Force $tests | Out-Null
$cp = (Get-GameClasspath) -join ';'
& "$Jdk/javac.exe" --release 25 -proc:none -cp $cp -d $tests "$Root/bot/tests/ObserverAudit.java"
if ($LASTEXITCODE) { throw 'Observer audit compilation failed' }
& "$Jdk/java.exe" -cp "$tests;$cp" pathtracing.ObserverAudit "$Output/observer-only.jar" 'D:/path-tracing-evidence/3.2d-3-A-baseline/pathtracing.jar'
if ($LASTEXITCODE) { throw 'Observer sites differ from frozen baseline' }
Write-Output "Observer-only artifact: $Output/observer-only.jar (no candidate controller or planner)"
