use std::{
    path::{Path, PathBuf},
    process::Command,
    string::String,
    boxed::Box,
    vec::Vec,
    vec,
    format,
};

pub fn build(
    dir: impl AsRef<Path>,
    args: &[&str],
    envs: &[(String, String)],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut build_args = vec!["build"];
    build_args.extend_from_slice(args);

    let envs_ext: Vec<(String, String)> = envs.iter().cloned().collect();

    let build_status = Command::new("cargo")
        .current_dir(dir)
        .args(build_args)
        .envs(envs_ext)
        .spawn()?
        .wait()?;

    if build_status.success() {
        return Ok(());
    }

    let err = match build_status.code() {
        Some(code) => format!("Build failed with code {}", code),
        None => String::from("Build failed due to signal"),
    };

    Err(Box::from(err))
}

pub fn target_output_file(build_args: &[&str], target_str: &str, crate_name: &str) -> PathBuf {
    let mut profile = None;
    for arg in build_args {
        if *arg == "--release" {
            profile = Some("release");
            break;
        }

        profile = arg.strip_prefix("--profile=");
        if profile.is_some() {
            break;
        }
    }

    let profile = profile.unwrap_or("debug");

    let path = format!("{}/target/{}/{}/{}", crate_name, target_str, profile, crate_name);
    PathBuf::from(path)
}

pub fn clean(dir: impl AsRef<Path>) -> Result<(), Box<dyn std::error::Error>> {
    let args = vec!["clean"];

    let clean_status = Command::new("cargo")
        .current_dir(dir)
        .args(args)
        .spawn()?
        .wait()?;

    if clean_status.success() {
        return Ok(());
    }

    let err = match clean_status.code() {
        Some(code) => format!("Build failed with code {}", code),
        None => String::from("Build failed due to signal"),
    };

    Err(Box::from(err))
}
