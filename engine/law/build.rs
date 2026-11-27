use std::{env, fs, path::{Path,PathBuf}};
fn files(path:&Path,out:&mut Vec<PathBuf>) {
    println!("cargo::rerun-if-changed={}",path.display());
    if path.is_dir() {for item in fs::read_dir(path).unwrap() {files(&item.unwrap().path(),out);}}
    else {out.push(path.to_owned());}
}
fn law_hash(root: &Path, data: &Path) {
    let engine=root.parent().unwrap();let mut inputs=Vec::new();
    for path in [root.join("src"),root.join("build.rs"),engine.join("world/src")] {files(&path,&mut inputs);}
    for entry in fs::read_dir(data).unwrap() {
        let path = entry.unwrap().path();
        if path.is_file() { files(&path, &mut inputs); }
    }
    inputs.sort();
    // Domain-separated FNV-128 streams, including input names and lengths.
    let mut hashes=[0x6c62272e07bb014262b821756295c58du128,0x6c62272e07bb014262b821756295c58cu128];
    let mut manifest=String::new();
    for path in inputs {
        let name=path.strip_prefix(engine).unwrap().to_string_lossy().replace('\\',"/");
        let content=fs::read(&path).unwrap();manifest.push_str(&name);manifest.push('\n');
        for byte in (name.len() as u64).to_le_bytes().into_iter().chain(name.bytes()).chain((content.len() as u64).to_le_bytes()).chain(content) {
            for h in &mut hashes {*h=(*h^byte as u128).wrapping_mul(0x0000000001000000000000000000013b);}
        }
    }
    let hash:Vec<u8>=hashes.into_iter().flat_map(u128::to_le_bytes).collect();
    let text=format!("pub const LAW_HASH:[u8;32]={hash:?};\npub const LAW_HASH_INPUTS:&str={manifest:?};\n");
    fs::write(PathBuf::from(env::var("OUT_DIR").unwrap()).join("law_hash.rs"),text).unwrap();
}
fn recordings(path: &Path) -> bool {
    if !path.exists() { return false; }
    fs::read_dir(path).expect("read fixtures").any(|entry| {
        let path=entry.expect("fixture entry").path();
        if path.is_dir() { return recordings(&path); }
        let name=path.file_name().unwrap().to_str().unwrap();
        name.ends_with(".jsonl") && !name.ends_with(".world.jsonl")
    })
}
fn main() {
    println!("cargo::rerun-if-env-changed=PATHTRACING_MC_VERSION");
    let root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let mut data = root.parent().unwrap().join("world/data");
    let mc = env::var("PATHTRACING_MC_VERSION").unwrap_or_default();
    if !mc.is_empty() && mc != "26.2" {
        assert!(mc.split_once('.').is_some_and(|(major, minor)| {
            [major, minor].iter().all(|s| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit()))
        }), "invalid Minecraft version {mc}");
        data.push(&mc);
    }
    assert!(data.join("version.txt").is_file(), "missing generated Minecraft tables at {}", data.display());
    println!("cargo::rustc-env=PATHTRACING_DATA_DIR={}", data.display());
    println!("cargo::rustc-env=PATHTRACING_LAW_VERSION={}", if mc.is_empty() { "26.2" } else { &mc });
    law_hash(&root, &data);
    let root=root.join("../../lab/fixtures");
    println!("cargo::rustc-check-cfg=cfg(has_recordings)");
    println!("cargo::rerun-if-changed={}",root.display());
    if recordings(&root) { println!("cargo::rustc-cfg=has_recordings"); }
    else { println!("cargo::warning=No recorder fixtures in lab/fixtures; replay is ignored (L0 has not run)"); }
}
