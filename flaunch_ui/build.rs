use cc;
use cmake::Config;

extern crate cbindgen;
use std::env;
use std::path::PathBuf;

static BUILD_TYPE: &'static str = "Release";

fn build_nanogui() {
    let mut conf = Config::new("../extern/nanogui");
    conf.profile(BUILD_TYPE);
    conf.define("NANOGUI_SHOW_WIDGET_BOUNDS", "");
    conf.build();

    if cfg!(target_family = "windows") {
        println!(
            "cargo:rustc-link-search=native={}\\build\\{}",
            conf.build().display(),
            BUILD_TYPE
        );

        println!("cargo:rustc-link-search=native=C:\\Program Files (x86)\\Windows Kits\\10\\Lib\\10.0.17763.0\\um\\x64");
        println!("cargo:rustc-link-lib=static=glu32");
    } else {
        println!(
            "cargo:rustc-link-search=native={}/build",
            conf.build().display()
        );
    }

    println!("cargo:rustc-link-lib=dylib=nanogui");
}

fn build_flaunch_ui() {
    // links against nanogui library.
    // includes and defines are exported from the nanogui cmake project.
    cc::Build::new()
        .file("src/flaunch_ui.cpp")
        //.file("src/image_loader.cpp")
        .file("../extern/lodepng/lodepng.cpp")
        .cpp(true)
        .opt_level(3)
        .flag_if_supported("-std=c++14")
        .flag_if_supported("-framework GLUT -framework OpenGL")
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

fn generate_c_bindings() {
    let crate_dir = format!(
        "{}/../flaunch_core",
        env::var("CARGO_MANIFEST_DIR").unwrap()
    );

    let package_name = env::var("CARGO_PKG_NAME").unwrap();
    let output_file = PathBuf::from(env::var("OUT_DIR").unwrap())
        .join(format!("{}.hpp", package_name))
        .display()
        .to_string();

    let config = cbindgen::Config {
        namespace: Some(String::from("ffi")),
        ..Default::default()
    };

    cbindgen::generate_with_config(&crate_dir, config)
        .unwrap()
        .write_to_file(&output_file);
}

// fn target_dir() -> PathBuf {
//     if let Ok(target) = env::var("CARGO_TARGET_DIR") {
//         PathBuf::from(target)
//     } else {
//         PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("target")
//     }
// }

fn main() {
    build_nanogui();
    generate_c_bindings();
    build_flaunch_ui();
}
