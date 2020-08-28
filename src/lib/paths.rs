use super::errors;
use std::{env, io, path::PathBuf};

const CONFIG_FILE_NAME: &str = "esbuild.config.json";

// Return the path of the config file, based on the passed string or by detecting it.
pub fn config_path(path: Option<&String>) -> Result<PathBuf, errors::ConfigPathError> {
    match path {
        Some(path) => {
            let esbuild_json = PathBuf::from(path);
            if esbuild_json.exists() {
                Ok(esbuild_json)
            } else {
                Err(errors::ConfigPathError::Io(io::Error::new(
                    io::ErrorKind::NotFound,
                    "The provided file doesn’t seem to exist.",
                )))
            }
        }
        None => Ok(detect_config_path()?),
    }
}

// Get the first ancestor directory containing a package.json
pub fn pkg_root_path() -> Result<PathBuf, errors::ConfigPathError> {
    let cwd = env::current_dir().map_err(errors::ConfigPathError::Io)?;

    for dir in cwd.ancestors() {
        if dir.join("package.json").exists() {
            return Ok(dir.to_path_buf());
        }
    }

    Err(errors::ConfigPathError::Io(io::Error::new(
        io::ErrorKind::NotFound,
        "No package.json found.",
    )))
}

// Detect the path of the config file from the current directory.
pub fn detect_config_path() -> Result<PathBuf, errors::ConfigPathError> {
    let cwd = env::current_dir().map_err(errors::ConfigPathError::Io)?;
    let local_esbuild_json = cwd.join(CONFIG_FILE_NAME);

    // Local esbuild.config.json
    if local_esbuild_json.exists() {
        return Ok(local_esbuild_json);
    }

    // Project root esbuild.config.json
    let local_esbuild_json = match pkg_root_path() {
        Ok(pkg_root) => pkg_root.join(CONFIG_FILE_NAME),
        Err(_) => {
            return Err(errors::ConfigPathError::Io(io::Error::new(
                io::ErrorKind::NotFound,
                [
                    "No ",
                    CONFIG_FILE_NAME,
                    " found in the current directory, and no project root found.",
                ]
                .concat(),
            )))
        }
    };

    if local_esbuild_json.exists() {
        Ok(local_esbuild_json)
    } else {
        Err(errors::ConfigPathError::Io(io::Error::new(
            io::ErrorKind::NotFound,
            [
                "No ",
                CONFIG_FILE_NAME,
                " found in the current directory nor in the project root.",
            ]
            .concat(),
        )))
    }
}
