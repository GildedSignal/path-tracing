$ErrorActionPreference='Stop'
$root='D:/path-tracing-wt/world'
$source="$root/engine/law/src/entity_contact.rs"
$original=[IO.File]::ReadAllText($source)
$needle='if distance < 0.01_f32 as f64 { return; }'
if (($original.Split(@($needle),[StringSplitOptions]::None).Count - 1) -ne 1) { throw 'mutant site count differs from one' }
$env:ENTITY_CONTACT_ORACLE="$root/oracles/1.9i-entity-contact/oracle.txt"
$env:CARGO_TARGET_DIR="$root/target"
try {
  [IO.File]::WriteAllText($source,$original.Replace($needle,'if distance <= 0.01_f32 as f64 { return; }'))
  Push-Location $root
  try { $ErrorActionPreference='Continue'; $output = & cargo test --manifest-path engine/law/Cargo.toml --test entity_contact direct_mojang_oracle -- --nocapture 2>&1 | Out-String; $code=$LASTEXITCODE; $ErrorActionPreference='Stop' } finally { Pop-Location }
  $output | Set-Content "$root/oracles/1.9i-entity-contact/mutant-output.txt"
  if ($code -eq 0 -or $output -notmatch 'first difference case 6 component 0') { throw "mutant did not fail at expected cutoff case: $output" }
  Write-Output 'PASS: exclusive-cutoff mutant caught at case 6 component 0'
} finally { [IO.File]::WriteAllText($source,$original) }
