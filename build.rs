fn main() {
    let target = std::env::var("TARGET").unwrap();

    if target.contains("windows-msvc") {
        println!("cargo:rustc-link-arg=/ENTRY:_start");
        println!("cargo:rustc-link-arg=/NODEFAULTLIB");
        println!("cargo:rustc-link-arg=/SUBSYSTEM:CONSOLE");
        println!("cargo:rustc-link-arg=/DEBUG:NONE");
    } else if target.contains("windows-gnu") {
        println!("cargo:rustc-link-arg=-Wl,--entry,_start");
        println!("cargo:rustc-link-arg=-nostdlib");
        println!("cargo:rustc-link-arg=-Wl,--subsystem,console");
    }
}
