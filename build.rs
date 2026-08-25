fn main() {
    println!("cargo:rustc-link-arg=/ENTRY:_start");
    println!("cargo:rustc-link-arg=/NODEFAULTLIB");
    println!("cargo:rustc-link-arg=/SUBSYSTEM:CONSOLE");
    println!("cargo:rustc-link-arg=/DEBUG:NONE");
}
