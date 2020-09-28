mod args;
pub mod errors;
mod paths;

use json;
use std::{fs, io, path};

pub enum ConfigFileType {
    ConfigJson,
    PackageJson,
}

pub fn esbuild_conf(args: Vec<String>) -> Result<String, errors::EsbuildConfigError> {
    let (config_path, config_file_type) =
        paths::config_path(args.get(1)).map_err(|_| errors::EsbuildConfigError::ConfigPathError)?;

    let config_content =
        read_json_content(config_path).map_err(|_| errors::EsbuildConfigError::ConfigParseError)?;

    parse_esbuild_config(config_content, config_file_type)
        .map_err(|_| errors::EsbuildConfigError::ConfigParseError)
}

pub fn read_json_content(path: path::PathBuf) -> Result<String, errors::EsbuildConfigError> {
    match fs::read_to_string(&path) {
        Ok(content) => Ok(content),
        Err(_) => Err(errors::EsbuildConfigError::Io(io::Error::new(
            io::ErrorKind::Other,
            [
                "Couldn’t read ",
                path.into_os_string()
                    .into_string()
                    .expect("The provided path couldn’t get read.")
                    .as_str(),
            ]
            .concat(),
        ))),
    }
}

// Parse the entire esbuild.config.json
pub fn parse_esbuild_config(
    content: String,
    config_file_type: ConfigFileType,
) -> Result<String, errors::ConfigParseError> {
    match json::parse(&content) {
        Ok(value) => match config_file_type {
            ConfigFileType::ConfigJson => args::args_from_config_json_value(value),
            ConfigFileType::PackageJson => args::args_from_package_json_value(value),
        }
        .map_err(|_| errors::ConfigParseError::InvalidConfigError),
        Err(_) => return Err(errors::ConfigParseError::InvalidConfigError),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_esbuild_config() {
        let config_json = r#"
            {
                "entry": "index.js",
                "a": true,
                "b": "abc",
                "c": ["def", "ghi"],
                "d": { "e": "jkl", "f": "mno" }
            }
        "#;
        assert_eq!(
            parse_esbuild_config(config_json.to_string(), ConfigFileType::ConfigJson).unwrap(),
            "--a --b=abc --c:def --c:ghi --d:e=jkl --d:f=mno index.js"
        );
        assert!(
            match parse_esbuild_config("true".to_string(), ConfigFileType::ConfigJson) {
                Ok(_) => false,
                Err(_) => true,
            }
        );

        let package_json = r#"
            {
                "esbuild": {
                    "entry": "index.js",
                    "a": true,
                    "b": "abc",
                    "c": ["def", "ghi"],
                    "d": { "e": "jkl", "f": "mno" }
                }
            }
        "#;
        assert_eq!(
            parse_esbuild_config(package_json.to_string(), ConfigFileType::PackageJson).unwrap(),
            "--a --b=abc --c:def --c:ghi --d:e=jkl --d:f=mno index.js"
        );
        assert!(
            match parse_esbuild_config("1".to_string(), ConfigFileType::PackageJson) {
                Ok(_) => false,
                Err(_) => true,
            }
        );
        assert!(
            match parse_esbuild_config("{}".to_string(), ConfigFileType::PackageJson) {
                Ok(_) => false,
                Err(_) => true,
            }
        );
    }
}
