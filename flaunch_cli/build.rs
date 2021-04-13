mod versioning;

fn main() {
    versioning::write_app_meta("flaunch_cli/src/app_meta.rs", "VERSION", "BUILD_DATE");
}
