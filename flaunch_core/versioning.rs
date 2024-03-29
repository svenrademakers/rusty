extern crate chrono;
extern crate git;

use chrono::prelude::*;
use regex::*;
use std::io::{Error, ErrorKind};
use std::path::{Path, PathBuf};

fn update_key_value(key: &str, value: &str, output: &str) -> Option<String> {
    let regex_string = format!("(.+){}(.+\").*(\".+)", key);
    if let Ok(re) = regex::Regex::new(&regex_string) {
        let result = re.replace(output, |caps: &Captures| {
            format!("{}{}{}{}{}", &caps[1], key, &caps[2], value, &caps[3])
        });
        return Some(result.to_string());
    }
    None
}

fn update_file(file: &Path, definitions: &Vec<(&str, &str)>) -> Result<(), Error> {
    let mut f = std::fs::read_to_string(file).expect(format!(" cannot read {:?}", file).as_str());
    for (key, value) in definitions {
        f = update_key_value(key, value, &f).ok_or(Error::new(
            ErrorKind::Other,
            format!("{} - {} not updated", key, value),
        ))?;
    }

    std::fs::write(file, f)
}

fn get_git_info() -> (PathBuf, String) {
    let curr_dir = std::env::current_dir().unwrap();
    let repo = git::Repository::discover(curr_dir).unwrap();

    let mut describe_options = git::DescribeOptions::new();
    describe_options.show_commit_oid_as_fallback(true);

    let mut describe_str = String::from("none");
    if let Ok(describe) = repo.describe(&describe_options) {
        describe_str = describe.format(None).unwrap();
    }

    let mut cwd = repo.path().to_path_buf();
    cwd.pop();
    (cwd, describe_str)
}

/// Updates a given file with git version number and current building date.
///
/// this function only works when run inside of a git repository.
///
/// # Arguments
///
/// * `rust_file` - filename to write to. file needs to be present in <project-root>/src
/// * `version_name` - name of the version variable
/// * `build_time_name` = name of the build_time variable
///
pub fn write_app_meta(rust_file: &str, version_name: &str, build_time_name: &str) {
    let (mut file, git_version) = get_git_info();
    file.push(rust_file);

    let mut definitions = Vec::new();
    definitions.push((version_name, &*git_version));

    let utc: DateTime<Utc> = Utc::now();
    let time_str = format!("{}-{}-{}", utc.day(), utc.month(), utc.year());
    definitions.push((build_time_name, &time_str));

    update_file(&file, &definitions).unwrap();
}
