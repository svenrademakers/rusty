extern crate git;
extern crate chrono;

use regex::*;
use std::path::{Path, PathBuf};
use std::io::{Error, ErrorKind};
use chrono::prelude::*;



fn update_key_value(key: &str, value: &str, output: & str) -> Option<()> {
    let regex_string = format!("(.+){}(.+\").*(\".+)", key);
    if let Ok(re) = regex::Regex::new(&regex_string){
        let result = re.replace(output, |caps: &Captures| {
            format!("{}{}{}{}{}", &caps[1], key, &caps[2], value, &caps[3])
        });
        assert_ne!(result, output);
       return Some(());
    }
    None
}

fn update_file(file: &Path, definitions: &Vec<(&str, &str)>) -> Result<(), Error> {
    let f = std::fs::read_to_string(file)?;
    for (key, value) in definitions {
        if update_key_value(key, value, &f).is_none() {
            return Err(Error::new(ErrorKind::Other, format!("{} - {} not updated", key, value)));
        }
    }
    
    std::fs::write(file, f)
}

fn get_git_info() -> (PathBuf, String) {
    let curr_dir = std::env::current_dir().unwrap();
    let repo = git::Repository::discover(curr_dir).unwrap();

    let mut describe_options = git::DescribeOptions::new();
    describe_options.show_commit_oid_as_fallback(true);
    let describe = repo.describe(&describe_options).unwrap();
    let describe_str = describe.format(None).unwrap();

    let mut cwd = repo.path().to_path_buf();
    cwd.pop();
    (cwd, describe_str)
}

pub fn write_app_meta(rust_file: &str, version_name : &str, build_time_name : &str)
{
    let (mut file, git_version) = get_git_info();
    file.push("src");
    file.push(rust_file);

    let mut definitions = Vec::new();
    definitions.push((version_name, &*git_version));

    let utc: DateTime<Utc> = Utc::now(); 
    let time_str = format!("{}-{}-{}", utc.year(), utc.month(), utc.day());
    definitions.push((build_time_name, &time_str));

    update_file(&file, &definitions).unwrap();
}