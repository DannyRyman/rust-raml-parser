extern crate yaml_rust;

use yaml_rust::{YamlLoader, Yaml};

pub fn parse(s: &str) -> Result<Raml, RamlErrors> {
    let ref yaml = load_yaml(s)?;

    error_if_incorrect_raml_comment(s)?;

    parse_document_root(yaml)
}

fn parse_document_root(yaml: &Yaml) -> Result<Raml, RamlErrors> {
    let mut raml_errors = RamlErrors::new();
    let mut title: Option<&str> = None;
    let mut version: Option<&str> = None;

    match yaml {
        &Yaml::Hash(ref h) => {
            for (k, v) in h {
                match k {
                    &Yaml::String(ref s) if s == "title" => {
                        title = v.as_str();
                    }
                    &Yaml::String(ref s) if s == "version" => {
                        version = v.as_str();
                    }
                    _ => {
                        // todo better error message
                        raml_errors.errors
                            .push(format!("Unexpected field at the document root"))
                    }
                }
            }
        }
        _ => raml_errors.errors.push(String::from("Unexpected YAML format")),
    }

    if title.is_none() {
        raml_errors.errors
            .push(String::from("Error parsing document root. Missing field: title"))
    }

    if raml_errors.errors.len() > 0 {
        Err(raml_errors)
    } else {
        Ok(Raml {
            title: title.unwrap().to_string(),
            version: version.map(|s| s.to_string()),
        })
    }
}

fn load_yaml(s: &str) -> Result<Yaml, RamlErrors> {
    let mut raml_errors = RamlErrors::new();
    let result = YamlLoader::load_from_str(s);
    match result {
        Ok(mut docs) => {
            if docs.len() == 0 {
                raml_errors.errors.push(String::from("Attempted to parse an empty document"));
                return Err(raml_errors);
            } else {
                Ok(docs.pop().unwrap())
            }
        }
        Err(scan_error) => {
            raml_errors.errors
                .push(format!("Invalid yaml: {}", scan_error));
            Err(raml_errors)
        } 
    }
}

fn error_if_incorrect_raml_comment(s: &str) -> Result<(), RamlErrors> {
    let first_line: &str = s.lines().next().unwrap_or_default().trim();
    if first_line != "#%RAML 1.0" {
        let mut raml_errors = RamlErrors::new();
        raml_errors.errors
            .push(String::from("Document must start with the following RAML comment line: \
                                #%RAML 1.0"));
        return Err(raml_errors);
    }
    Ok(())
}

#[derive(Debug)]
pub struct Raml {
    title: String,
    version: Option<String>,
}

#[derive(Debug)]
pub struct RamlErrors {
    errors: Vec<String>,
}

impl RamlErrors {
    pub fn new() -> RamlErrors {
        let errs = Vec::new();
        RamlErrors { errors: errs }
    }
}

#[test]
fn error_on_empty_document() {
    let s = "";
    let result = parse(s);
    assert_eq!(result.is_err(), true);
    let errors = result.err().unwrap().errors;
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0], "Attempted to parse an empty document");
}

#[test]
fn error_for_missing_version_comment() {
    let s = "title: Some API";
    let result = parse(s);
    assert_eq!(result.is_err(), true);
    let errors = result.err().unwrap().errors;
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0],
               "Document must start with the following RAML comment line: #%RAML 1.0");
}

#[test]
fn error_for_invalid_yaml() {
    let s = "%%invalid";
    let result = parse(s);
    assert_eq!(result.is_err(), true);
    let errors = result.err().unwrap().errors;
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0],
               "Invalid yaml: while scanning a directive, could not find expected directive name \
                at line 1 column 2");
}

#[test]
fn loads_the_title() {
    let s = "#%RAML 1.0
    title: Some API
    ";
    let result = parse(s);
    println!("result is error: {}", result.is_err());
    assert_eq!(result.is_ok(), true);
    let raml = result.ok().unwrap();
    assert_eq!("Some API", raml.title);
}

// todo fail when mandatory title is not set
#[test]
fn error_for_missing_title() {
    let s = "#%RAML 1.0
    version: v1";
    let result = parse(s);
    assert_eq!(result.is_err(), true);
    let errors = result.err().unwrap().errors;
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0],
               "Error parsing document root. Missing field: title");
}

#[test]
fn loads_the_version() {
    let s = "#%RAML 1.0
    title: Some API
    version: v1
    ";
    let result = parse(s);
    println!("result is error: {}", result.is_err());
    assert_eq!(result.is_ok(), true);
    let raml = result.ok().unwrap();
    assert_eq!("Some API", raml.title);
    assert_eq!("v1", raml.version.unwrap());
}
