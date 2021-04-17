use cc;
use cmake::Config;
extern crate bindgen;

use std::env;
use std::path::PathBuf;

static BUILD_TYPE: &'static str = "Release";

fn main() {
    // build nanogui external C++ gui library
    let mut conf = Config::new("../extern/nanogui");
    conf.profile(BUILD_TYPE);
    conf.build();
    println!(
        "cargo:rustc-link-search=native={}\\build\\{}",
        conf.build().display(),
        BUILD_TYPE
    );

    // build Flaunch C++ UI
    // links against nanogui library.
    // includes and defines are exported from the nanogui cmake project.
    cc::Build::new()
        .file("src/flaunch_ui.cpp")
        .cpp(true)
        .include(format!(
            "{}/../extern/nanogui/ext/glad/include",
            env!("CARGO_MANIFEST_DIR")
        ))
        .include(format!(
            "{}/../extern/nanogui/ext/eigen",
            env!("CARGO_MANIFEST_DIR")
        ))
        .include(format!(
            "{}/../extern/nanogui/ext/glfw/include",
            env!("CARGO_MANIFEST_DIR")
        ))
        .include(format!(
            "{}/../extern/nanogui/ext/nanovg/src",
            env!("CARGO_MANIFEST_DIR")
        ))
        .include(format!(
            "{}/../extern/nanogui/include",
            env!("CARGO_MANIFEST_DIR")
        ))
        .define("NVG_SHARED", None)
        .define("GLAD_GLAPI_EXPORT", None)
        .define("NANOGUI_GLAD", None)
        .compile("flaunch_ui");

    println!("cargo:rerun-if-changed=incl/flaunch_ui.hpp");
    println!("cargo:rerun-if-changed=src/flaunch_ui.cpp");
    println!("cargo:rustc-link-lib=static=flaunch_ui");
    println!("cargo:rustc-link-lib=static=nanogui");

    // generate bindings for ui framework
    println!("cargo:rustc-env=BINDGEN_EXTRA_CLANG_ARGS=-x c++ -std=c++11");
    let bindings = bindgen::Builder::default()
        .header("incl/flaunch_ui.hpp")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .enable_cxx_namespaces()
        .opaque_type("std::.*")
        .allowlist_type("std::string|void.*")
        .allowlist_function("ui.*")
        .default_enum_style(bindgen::EnumVariation::Rust {
            non_exhaustive: false,
        })
        .generate()
        .unwrap();

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
