use cc;
use cmake::Config;
extern crate bindgen;

use std::env;
use std::path::PathBuf;

fn generate_rust_bindings() {
    println!("cargo:rustc-env=BINDGEN_EXTRA_CLANG_ARGS=-x c++ -std=c++14");
    let bindings = bindgen::Builder::default()
        .header("../flaunch_ui/incl/flaunch_ui.hpp")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .enable_cxx_namespaces()
        .opaque_type("std::.*")
        .allowlist_type("std::string|void.*")
        .allowlist_function("ui.*")
        .default_enum_style(bindgen::EnumVariation::NewType { is_bitfield: false })
        .generate()
        .unwrap();

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}

fn main() {
    generate_rust_bindings();
}
