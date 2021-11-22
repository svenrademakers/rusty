use std::{env, path::PathBuf, process::Command};

fn main() {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("resources");

    let status = Command::new("glib-compile-resources")
        .current_dir(path.as_path())
        .arg(format!("--sourcedir={}", path.to_string_lossy()))
        .arg("app.gresource.xml")
        .status()
        .unwrap();

    assert!(status.success());
}
