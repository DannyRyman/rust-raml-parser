extern crate yaml_rust;

use yaml_rust::{YamlLoader, Yaml};

pub fn parse(s: &str) -> Result<Raml, RamlErrors> {
    let ref docs = load_yaml(s)?;

    error_if_incorrect_raml_comment(s)?;

    let ref yaml = docs[0];

    let title = yaml["title"].as_str().unwrap();
    Ok(Raml { title: title.to_string() })
}

fn load_yaml(s: &str) -> Result<Vec<Yaml>, RamlErrors> {
    let mut raml_errors = RamlErrors::new();
    let result = YamlLoader::load_from_str(s);
    match result {
        Ok(docs) => {
            if docs.len() == 0 {
                raml_errors.errors.push(String::from("Attempted to parse an empty document"));
                return Err(raml_errors);
            } else {
                Ok(docs)
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
fn must_error_on_empty_document() {
    let s = "";
    let result = parse(s);
    assert_eq!(result.is_err(), true);
    let errors = result.err().unwrap().errors;
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0], "Attempted to parse an empty document");
}

#[test]
fn must_start_with_version_comment() {
    let s = "title: Some API";
    let result = parse(s);
    assert_eq!(result.is_err(), true);
    let errors = result.err().unwrap().errors;
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0],
               "Document must start with the following RAML comment line: #%RAML 1.0");
}

#[test]
fn must_error_for_invalid_yaml() {
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
    println!("test");
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
