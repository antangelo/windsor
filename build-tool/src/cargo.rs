use std::{
    boxed::Box,
    format,
    path::{Path, PathBuf},
    process::Command,
    string::String,
    vec,
    vec::Vec,
};

pub fn build(
    dir: impl AsRef<Path>,
    args: &[&str],
    envs: &[(String, String)],
    toolchain: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let sys_envs: Vec<(String, String)> = std::env::vars().filter(|(e, _)| {
        for (var, _) in envs.iter() {
            if var == e {
                return false;
            }
        }

        true
    }).collect();

    let mut envs_ext: Vec<(String, String)> = envs.iter().cloned().collect();
    envs_ext.extend(sys_envs.into_iter());

    let toolchain = toolchain.as_ref().map(|s| s.as_str()).unwrap_or("nightly");
    let toolchain = format!("+{}", toolchain);
    let mut build_args = vec![&toolchain, "build"];
    build_args.extend_from_slice(args);

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

    let path = format!(
        "{}/target/{}/{}/{}",
        crate_name, target_str, profile, crate_name
    );
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
