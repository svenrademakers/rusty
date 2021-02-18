mod versioning;

fn main() {
    versioning::write_app_meta("hypter_cli/src/app_meta.rs", "VERSION", "BUILD_DATE");
}
