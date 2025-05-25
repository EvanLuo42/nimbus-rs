use std::env;
use std::fmt::Debug;
use std::path::PathBuf;

fn main() {
    let dotnet_include_path = env::var("DOTNET_INCLUDE").unwrap();
    let bindings = bindgen::Builder::default()
        .header(format!("{}/hostfxr.h", dotnet_include_path))
        .header(format!("{}/coreclr_delegates.h", dotnet_include_path))
        .header(format!("{}/nethost.h", dotnet_include_path))
        .clang_arg(format!("-I{}", dotnet_include_path))
        .prepend_enum_name(true)
        .generate()
        .unwrap();

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("corehost.rs"))
        .unwrap()
}
