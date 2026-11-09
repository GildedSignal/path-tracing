# Free-glide oracle and candidate comparison

The staged 26.2 jar is the oracle. The generator invokes `LivingEntity.updateFallFlyingMovement` before collision, after setting the probe's rotation through the inherited entity setters. The probe holds effective gravity fixed at the vanilla player default. The results cover free glide with no status effects. Entry, exit, collision, durability and boost are outside this comparison.

From this worktree, generate both JVM runs and compare the Rust candidate and its negative control. Use an unused evidence directory.

```powershell
$evidence = 'D:/path-tracing-evidence/elytra-free-glide'
$env:RUSTUP_HOME = 'D:/rust/rustup'
$env:CARGO_HOME = 'D:/rust/cargo'
& .\jobs\1.9b-elytra-oracle\run.ps1 -Output $evidence
& D:/rust/cargo/bin/rustc.exe --edition 2024 -O .\jobs\1.9b-elytra-oracle\compare.rs -o "$evidence/compare.exe"
& "$evidence/compare.exe" "$evidence/jit.csv"
& "$evidence/compare.exe" "$evidence/interpreter.csv"
& "$evidence/compare.exe" "$evidence/jit.csv" table-cosine
```

Compare every output component by raw bits. The candidate passes when every row matches. The control substitutes Minecraft's float cosine table for the JVM cosine in the lift term and must exit nonzero. Preserve the first difference verbatim.
