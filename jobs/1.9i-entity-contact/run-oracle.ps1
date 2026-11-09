$ErrorActionPreference='Stop'
$root='D:/path-tracing-wt/world'
$job="$root/jobs/1.9i-entity-contact"
$jdk='D:/path-tracing-deps/data/java/java-runtime-epsilon/bin'
$jar='D:/path-tracing-lab/server/versions/26.2/server-26.2.jar'
$libs=Get-ChildItem -Path 'D:/path-tracing-lab/server/libraries' -Filter '*.jar' -Recurse | Sort-Object FullName | ForEach-Object FullName
$cp=(@($jar)+@($libs)) -join ';'
New-Item -ItemType Directory -Force "$job/classes" | Out-Null
& "$jdk/javac.exe" -cp $cp -d "$job/classes" "$job/Oracle.java"
if ($LASTEXITCODE -ne 0) { throw 'javac failed' }
 $raw = & "$jdk/java.exe" -cp "$job/classes;$cp" Oracle
if ($LASTEXITCODE -ne 0) { throw 'java failed' }
$raw | Where-Object { $_ -match '^\d+ [0-9a-f]{16} ' } | Set-Content "$job/oracle.txt"
if ((Get-Content "$job/oracle.txt").Count -ne 170) { throw 'missing oracle rows' }
$env:ENTITY_CONTACT_ORACLE="$job/oracle.txt"
$env:CARGO_TARGET_DIR='D:/path-tracing-wt/world/target'
Push-Location $root
try { cargo test --manifest-path engine/law/Cargo.toml --test entity_contact; if ($LASTEXITCODE -ne 0) { throw 'comparison failed' } } finally { Pop-Location }
