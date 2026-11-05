use std::{env, fs, path::Path};
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
    let root=Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap()).join("../../lab/fixtures");
    println!("cargo::rustc-check-cfg=cfg(has_recordings)");
    println!("cargo::rerun-if-changed={}",root.display());
    if recordings(&root) { println!("cargo::rustc-cfg=has_recordings"); }
    else { println!("cargo::warning=No recorder fixtures in lab/fixtures; replay is ignored (L0 has not run)"); }
}
