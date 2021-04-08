use cmake;

fn main() {
    let dst = cmake::build("nanocui");

    println!("cargo:rustc-link-search=native={}", dst.display());
    println!("cargo:rustc-link-lib=static=nanocui");
}
