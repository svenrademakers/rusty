mod versioning;

fn main() {
    versioning::write_app_meta("app_meta.rs", "VERSION", "BUILD_DATE");
}
