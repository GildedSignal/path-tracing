# Compile a smoke check against the unmodified wt/law parser copied into ignored scratch output.
param([string]$Rustc = 'D:\rust\rustup\toolchains\stable-x86_64-pc-windows-gnu\bin\rustc.exe')
$ErrorActionPreference = 'Stop'
$scratch = Join-Path $PSScriptRoot ('..\results\json-smoke-' + [Guid]::NewGuid().ToString('N'))
$null = New-Item -ItemType Directory -Force $scratch
$parser = & git -C (Join-Path $PSScriptRoot '..\..\..') show wt/law:engine/law/tests/support/json.rs
if ($LASTEXITCODE) { throw 'Cannot read wt/law JSON parser.' }
[IO.File]::WriteAllLines((Join-Path $scratch 'json.rs'), [string[]]$parser, [Text.UTF8Encoding]::new($false))
$source = @'
mod json;
use json::{Object, numbers};
fn main() {
    let line = r#"{"kind":"block","at":[4112,-61,4100],"boxes":[[0,0,0,1,1,1]],"aboveBoxes":[[0,0,0,1,1,1]],"belowBoxes":[[0,0,0,1,1,1]],"sneakBoxes":[[0,0,0,1,1,1]],"insideBoxes":[[0,0,0,1,1,1]],"friction":0.6,"frictionBits":"3f19999a","speedFactor":1.0,"jumpFactor":1.0,"bounce":0.0,"climbable":false,"scaffolding":false,"slime":false,"fence":false,"wallOrGate":false,"suppressesBounce":false,"resetsFallDistance":false,"waterBlock":false,"air":false,"fluid":"empty","fluidHeight":0.0,"flow":{"x":0.0,"xBits":"0000000000000000","y":0.0,"yBits":"0000000000000000","z":0.0,"zBits":"0000000000000000"},"inside":"none","fallOnMultiplier":1.0}"#;
    let block = Object::parse(line);
    assert_eq!(block.string("kind"), "block");
    assert_eq!(numbers(block.required("at")), [4112.0, -61.0, 4100.0]);
    for key in ["boxes", "aboveBoxes", "belowBoxes", "sneakBoxes", "insideBoxes"] {
        assert_eq!(numbers(block.required(key)), [0.0, 0.0, 0.0, 1.0, 1.0, 1.0]);
    }
    assert_eq!(block.f32("friction").to_bits(), 0.6f32.to_bits());
    assert_eq!(block.f32("speedFactor"), 1.0);
    assert_eq!(block.f32("jumpFactor"), 1.0);
    assert_eq!(block.f32("fallOnMultiplier"), 1.0);
    assert_eq!(block.f32("bounce"), 0.0);
    for key in ["climbable", "scaffolding", "slime", "fence", "wallOrGate", "suppressesBounce", "resetsFallDistance", "waterBlock", "air"] {
        assert!(!block.boolean(key));
    }
    assert_eq!(block.string("fluid"), "empty");
    assert_eq!(block.f32("fluidHeight"), 0.0);
    assert_eq!(block.string("inside"), "none");
    for axis in ["x", "y", "z"] { assert_eq!(block.object("flow").f64(axis).to_bits(), 0); }
    let tick = Object::parse(r#"{"t":0,"ticksFrozen":14,"speedModifiers":[["minecraft:powder_snow",-0.005,0]],"collisionSteps":[{"requested":{"xBits":"3fb999999999999a","yBits":"bfb41205bc01a36e","zBits":"3fb999999999999a"},"result":{"xBits":"3fb999999999999a","yBits":"0000000000000000","zBits":"0000000000000000"}}]}"#);
    assert_eq!(tick.number("ticksFrozen"), 14.0);
    assert!(tick.required("collisionSteps").contains("\"xBits\":\"3fb999999999999a\""));
    assert!(tick.required("speedModifiers").contains("minecraft:powder_snow"));
    let end = Object::parse(r#"{"kind":"serverEnd","atRest":true,"serverTickOffset":4,"serverTickOffsetMin":4,"serverTickOffsetMax":4,"health":20,"healthBits":"41a00000","foodLevel":20,"saturationLevel":5,"exhaustion":0.75,"foodTickTimer":0}"#);
    assert_eq!(end.string("kind"), "serverEnd");
    assert!(end.boolean("atRest"));
    assert_eq!(end.number("serverTickOffset"), 4.0);
    assert_eq!(end.number("foodTickTimer"), 0.0);
    println!("PASS hand-written export block through wt/law Object, numeric boxes and raw float/vector parsing");
}
'@
[IO.File]::WriteAllText((Join-Path $scratch 'main.rs'), $source, [Text.UTF8Encoding]::new($false))
$exe = Join-Path $scratch 'json-smoke.exe'
& $Rustc --edition=2024 --crate-name fixture_json_smoke (Join-Path $scratch 'main.rs') -o $exe
if ($LASTEXITCODE) { throw 'Rust parser smoke check did not compile.' }
& $exe
if ($LASTEXITCODE) { throw 'Rust parser smoke check failed.' }
