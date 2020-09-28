use super::errors;
use json;
use snailquote;

// Get the esbuild arguments from the esbuild.config.json data structure
pub fn args_from_config_json_value(
    json: json::JsonValue,
) -> Result<String, errors::ConfigParseError> {
    let mut args: Vec<String> = vec![];
    let mut entries: Vec<String> = vec![];
    let mut options: Vec<String> = vec![];

    if !json.is_object() {
        return Err(errors::ConfigParseError::JsonError(json::Error::WrongType(
            String::from("The JSON main type must be an object."),
        )));
    }

    for (key, value) in json.entries() {
        if key == "entry" {
            match entries_from_config(value) {
                Some(result) => entries = result,
                None => (),
            }
            continue;
        }
        match option_from_config(key, value) {
            Some(param) => options.push(param),
            None => (),
        }
    }

    args.append(&mut options);
    args.append(&mut entries);
    Ok(args.join(" "))
}

// Get the esbuild arguments from the data structure of the package.json esbuild field
pub fn args_from_package_json_value(
    json: json::JsonValue,
) -> Result<String, errors::ConfigParseError> {
    if !json.is_object() {
        return Err(errors::ConfigParseError::JsonError(json::Error::WrongType(
            String::from("The package.json seems malformed."),
        )));
    }
    args_from_config_json_value(json["esbuild"].clone())
}

// Get the entries in the config file
pub fn entries_from_config(value: &json::JsonValue) -> Option<Vec<String>> {
    if value.is_string() {
        return Some(vec![quote_value(value.as_str().unwrap())]);
    }
    if value.is_array() {
        let entries: Vec<String> = value
            .members()
            .filter_map(|entry| match entry.as_str() {
                Some(value) => Some(quote_value(value)),
                None => None,
            })
            .collect();
        return match entries.is_empty() {
            false => Some(entries),
            true => None,
        };
    }
    None
}

// Parse a single config value from esbuild.config.json
pub fn option_from_config(key: &str, value: &json::JsonValue) -> Option<String> {
    if value.is_boolean() {
        return option_from_bool(key, value);
    }
    if value.is_string() {
        return option_from_string(key, value);
    }
    if value.is_array() {
        return option_from_array(key, value);
    }
    if value.is_object() {
        return option_from_object(key, value);
    }
    None
}

// Parse a bool config value
pub fn option_from_bool(key: &str, value: &json::JsonValue) -> Option<String> {
    match value.as_bool() {
        Some(value) => {
            if value {
                Some(["--", key].concat())
            } else {
                None
            }
        }
        None => None,
    }
}

// Parse a string config value
pub fn option_from_string(key: &str, value: &json::JsonValue) -> Option<String> {
    match value.as_str() {
        Some(value) => Some(["--", key, "=", &quote_value(value)].concat()),
        None => None,
    }
}

// Parse an object config value
pub fn option_from_object(key: &str, value: &json::JsonValue) -> Option<String> {
    let mut options: Vec<String> = vec![];

    for (k, v) in value.entries() {
        match v.as_str() {
            Some(value) => options.push(["--", key, ":", k, "=", &quote_value(value)].concat()),
            None => (),
        }
    }

    if options.len() > 0 {
        Some(options.join(" "))
    } else {
        None
    }
}

// Parse an array config value
pub fn option_from_array(key: &str, value: &json::JsonValue) -> Option<String> {
    let mut options: Vec<String> = vec![];

    for param_value in value.members() {
        match param_value.as_str() {
            Some(value) => options.push(["--", key, ":", &quote_value(value)].concat()),
            None => (),
        }
    }

    if options.len() > 0 {
        Some(options.join(" "))
    } else {
        None
    }
}

// Quote a value if it contains a space
pub fn quote_value(value: &str) -> String {
    let value = snailquote::escape(&value).to_string();
    if value == "" {
        String::from("''")
    } else {
        value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_args_from_json_value() {
        let value = json::parse(
            r#"{
                "entry": "index.js",
                "a": true,
                "b": "abc",
                "c": ["def", "ghi"],
                "d": { "e": "jkl", "f": "mno" }
            }"#,
        )
        .unwrap();
        assert_eq!(
            args_from_config_json_value(value).unwrap(),
            "--a --b=abc --c:def --c:ghi --d:e=jkl --d:f=mno index.js"
        );
    }

    #[test]
    fn test_entries_from_config() {
        let value = json::parse("\"path/to/some/file.js\"").unwrap();
        let entries = entries_from_config(&value).unwrap();
        assert_eq!(entries[0], "path/to/some/file.js");

        let value = json::parse("[\"path/to/some/file.js\"]").unwrap();
        let entries = entries_from_config(&value).unwrap();
        assert_eq!(entries[0], "path/to/some/file.js");

        let value = json::parse(
            r#"[
                "path/to/some/file.js",
                "./path with spaces.js",
                true
            ]"#,
        )
        .unwrap();
        let entries = entries_from_config(&value).unwrap();
        assert_eq!(entries[0], "path/to/some/file.js");
        assert_eq!(entries[1], "'./path with spaces.js'");
        assert!(entries.get(2).is_none());

        let value = json::parse("[]").unwrap();
        assert!(entries_from_config(&value).is_none());

        let value = json::parse("true").unwrap();
        assert!(entries_from_config(&value).is_none());
    }

    #[test]
    fn test_option_from_config() {
        let value = json::parse("true").unwrap();
        assert!(!option_from_config("name", &value).is_none());

        let value = json::parse("false").unwrap();
        assert!(option_from_config("name", &value).is_none());

        let value = json::parse("\"a\"").unwrap();
        assert!(!option_from_config("name", &value).is_none());

        let value = json::parse("[\"a\"]").unwrap();
        assert!(!option_from_config("name", &value).is_none());

        let value = json::parse("{\"a\": \"abc\"}").unwrap();
        assert!(!option_from_config("name", &value).is_none());

        let value = json::parse("null").unwrap();
        assert!(option_from_config("name", &value).is_none());
    }

    #[test]
    fn test_option_from_bool() {
        let value = json::parse("true").unwrap();
        assert_eq!(option_from_bool("name", &value).unwrap(), "--name");

        let value = json::parse("false").unwrap();
        assert!(option_from_bool("name", &value).is_none());

        // Wrong types get ignored
        let value = json::parse("1").unwrap();
        assert!(option_from_bool("name", &value).is_none());
    }

    #[test]
    fn test_option_from_string() {
        let value = json::parse("\"a\"").unwrap();
        assert_eq!(option_from_string("name", &value).unwrap(), "--name=a");

        // Wrong types get ignored
        let value = json::parse("1").unwrap();
        assert!(option_from_string("name", &value).is_none());

        // Empty value
        let value = json::parse("\"\"").unwrap();
        assert_eq!(option_from_string("name", &value).unwrap(), "--name=''");
    }

    #[test]
    fn test_option_from_object() {
        let value = json::parse("{ \"a\": \"abc\", \"b\": \"def\" }").unwrap();
        assert_eq!(
            option_from_object("name", &value).unwrap(),
            "--name:a=abc --name:b=def"
        );

        let value = json::parse("{}").unwrap();
        assert!(option_from_object("name", &value).is_none());

        // Wrong types in the object get ignored
        let value = json::parse("{ \"a\": \"abc\", \"b\": 123 }").unwrap();
        assert_eq!(option_from_object("name", &value).unwrap(), "--name:a=abc");
    }

    #[test]
    fn test_option_from_array() {
        let value = json::parse("[\"a\", \"b\", \"c\"]").unwrap();
        assert_eq!(
            option_from_array("name", &value).unwrap(),
            "--name:a --name:b --name:c"
        );

        // Empty arrays
        let value = json::parse("[]").unwrap();
        assert!(option_from_array("name", &value).is_none());

        // Wrong types in the array get ignored
        let value = json::parse("[\"a\", 1, \"b\"]").unwrap();
        assert_eq!(
            option_from_array("name", &value).unwrap(),
            "--name:a --name:b"
        );
    }

    #[test]
    fn test_quote_value() {
        assert_eq!(quote_value("value"), "value");

        // Having a space should return the value with quotes
        assert_eq!(quote_value("with space"), "'with space'");

        // Having a quote should return the value with quotes
        assert_eq!(quote_value("with\"quote"), "'with\"quote'");
        assert_eq!(quote_value("with'quote"), "\"with'quote\"");
    }
}
