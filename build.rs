use std::env;

fn main() {
    let target = env::var("TARGET").unwrap();
    if target.starts_with("x86_64") {
        println!("cargo:rerun-if-changed=src/arch/x86_64/link.ld");
        println!("cargo:rustc-link-arg=-Tsrc/arch/x86_64/link.ld");
        println!("cargo:rustc-cfg=x86_64");
    }
    println!("cargo:rerun-if-changed=build.rs");
}
