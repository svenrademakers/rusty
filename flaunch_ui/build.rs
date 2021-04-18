use cc;
use cmake::Config;
extern crate bindgen;

use std::env;
use std::path::PathBuf;

static BUILD_TYPE: &'static str = "Release";

fn build_nanogui() {
    let mut conf = Config::new("../extern/nanogui");
    conf.profile(BUILD_TYPE);
    conf.define("NANOGUI_SHOW_WIDGET_BOUNDS", "");
    conf.build();
    println!(
        "cargo:rustc-link-search=native={}\\build\\{}",
        conf.build().display(),
        BUILD_TYPE
    );

    println!("cargo:rustc-link-search=native=C:\\Program Files (x86)\\Windows Kits\\10\\Lib\\10.0.17763.0\\um\\x64");
    println!("cargo:rustc-link-lib=static=nanogui");
    println!("cargo:rustc-link-lib=static=glu32");
}

fn build_flaunch_ui() {
    // links against nanogui library.
    // includes and defines are exported from the nanogui cmake project.
    cc::Build::new()
        .file("src/flaunch_ui.cpp")
        .file("src/image_loader.cpp")
        .file("../extern/lodepng/lodepng.cpp")
        .cpp(true)
        .include("../extern/nanogui/ext/glfw/deps")
        .include("../extern/nanogui/ext/eigen")
        .include("../extern/nanogui/ext/glfw/include")
        .include("../extern/nanogui/ext/nanovg/src")
        .include("../extern/nanogui/include")
        .include("incl")
        .include("../extern/lodepng")
        .define("NVG_SHARED", None)
        .define("GLAD_GLAPI_EXPORT", None)
        .define("NANOGUI_GLAD", None)
        .compile("flaunch_ui");

    println!("cargo:rerun-if-changed=incl/flaunch_ui.hpp");
    println!("cargo:rerun-if-changed=src/flaunch_ui.cpp");
    println!("cargo:rerun-if-changed=incl/imageloader.hpp");
    println!("cargo:rerun-if-changed=src/imageloader.cpp");
    println!("cargo:rustc-link-lib=static=flaunch_ui");
}

fn generate_rust_bindings() {
    // generate bindings for ui framework
    println!("cargo:rustc-env=BINDGEN_EXTRA_CLANG_ARGS=-x c++ -std=c++14");
    let bindings = bindgen::Builder::default()
        .header("incl/flaunch_ui.hpp")
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
    build_nanogui();
    build_flaunch_ui();
    generate_rust_bindings();
}
