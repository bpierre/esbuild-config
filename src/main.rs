mod lib;

use lib::errors::EsbuildConfigError;
use std::env;

fn main() {
    match lib::esbuild_conf(env::args().collect()) {
        Ok(value) => println!("{}", value),
        Err(err) => match err {
            EsbuildConfigError::ConfigParseError => {
                eprintln!("The configuration file is invalid.");
            }
            EsbuildConfigError::ConfigPathError => {
                eprintln!("Couldn’t find or open the esbuild configuration file.");
            }
            _ => {
                eprintln!("Error: {}", err);
            }
        },
    }
}
