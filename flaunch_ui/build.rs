use cmake::Config;
extern crate bindgen;

use std::env;
use std::path::PathBuf;

static BUILD_TYPE : &'static str = "RelWithDebInfo";

fn main() {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    let mut conf = Config::new(".");
    conf.profile(BUILD_TYPE);
    conf.build_target("flaunch_ui");
    println!("cargo:rerun-if-changed=incl/flaunch_ui.hpp");
    println!("cargo:rerun-if-changed=src/flaunch_ui.cpp");

    println!(
        "cargo:rustc-link-search=native={}\\build\\{}",
        conf.build().display(), BUILD_TYPE
    );
    println!("cargo:rustc-link-lib=static=flaunch_ui");
    println!("cargo:rustc-link-search=native={}\\build\\extern\\nanogui\\RelWithDebInfo", out_path.to_str().unwrap());
    println!("cargo:rustc-link-search=native=C:\\Program Files (x86)\\Windows Kits\\10\\Lib\\10.0.17763.0\\um\\x64");
    println!("cargo:rustc-link-lib=static=nanogui");
    println!("cargo:rustc-link-lib=static=ComDlg32");

    println!("cargo:rustc-env=BINDGEN_EXTRA_CLANG_ARGS=-x c++ -std=c++11");
    // generate bindings for ui framework
    let bindings = bindgen::Builder::default()
    .header("incl/flaunch_ui.hpp")
    .parse_callbacks(Box::new(bindgen::CargoCallbacks))
    .enable_cxx_namespaces()
    .opaque_type("std::.*")
    .allowlist_type("std::string")
    .allowlist_function("ui.*")
    .generate()
    .unwrap();

    bindings
        .write_to_file(out_path.join("flaunch_ui_bindings.rs"))
        .expect("Couldn't write bindings!");
}
