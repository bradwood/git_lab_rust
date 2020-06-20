use std::env;
use std::fs;
use std::path::Path;

include!("man/man.rs");

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let man_path = Path::new(&out_dir).join("git-lab.1");
    fs::write(
        &man_path,
        man()
    ).unwrap();

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=man/man.rs");
}

