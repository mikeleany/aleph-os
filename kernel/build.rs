
fn main() {
    let target = std::env::var("TARGET").expect("determine target");
    println!("cargo:rerun-if-changed=custom-targets/{}.json", target);
    println!("cargo:rerun-if-changed=aleph-os-kernel.ld");
}
