use cmake::Config;

fn main() {
    let mut conf = Config::new("nanocui");
    conf.profile("RelWithDebInfo");
    conf.build_target("nanocui");

    println!(
        "cargo:rustc-link-search=native={}\\build\\RelWithDebInfo",
        conf.build().display()
    );
    println!("cargo:rustc-link-lib=static=nanocui");
    println!("cargo:rustc-link-search=native=C:\\Users\\sven\\Documents\\GitHub\\rusty\\target\\debug\\build\\nanogui_wrapper-f62b0c43b5b0c68e\\out\\build\\extern\\nanogui\\RelWithDebInfo");
    println!("cargo:rustc-link-search=native=C:\\Program Files (x86)\\Windows Kits\\10\\Lib\\10.0.17763.0\\um\\x64");
    println!("cargo:rustc-link-lib=static=nanogui");
    println!("cargo:rustc-link-lib=static=ComDlg32");
}
