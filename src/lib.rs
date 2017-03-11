extern crate yaml_rust;

use yaml_rust::{YamlLoader, Yaml};

#[derive(Debug)]
struct Raml {
    title: String,
}

#[derive(Debug)]
struct RamlErrors {
    errors: Vec<String>,
}

impl RamlErrors {
    pub fn new() -> RamlErrors {
        let errs = Vec::new();
        RamlErrors { errors: errs }
    }
}

fn parse(s: &str) -> Result<Raml, RamlErrors> {
    let mut raml_errors = RamlErrors::new();
    let ref docs = load_yaml(s)?;
    if docs.len() == 0 {
        raml_errors.errors.push(String::from("Attempted to parse an empty document"));
        return Err(raml_errors);
    }
    let first_line: &str = s.lines().next().unwrap_or_default().trim();
    println!("first_line: {}", first_line);
    if first_line != "#%RAML 1.0" {
        raml_errors.errors
            .push(String::from("Document must start with the following RAML comment line: \
                                #%RAML 1.0"));
        return Err(raml_errors);
    }
    let ref doc = docs[0];
    let title = doc["title"].as_str().unwrap();
    println!("title >>>>>> {0}", title);
    Ok(Raml { title: title.to_string() })
}

fn load_yaml(s: &str) -> Result<Vec<Yaml>, RamlErrors> {
    let result = YamlLoader::load_from_str(s);
    match result {
        Ok(yaml) => Ok(yaml),
        Err(scan_error) => {
            let mut raml_errors = RamlErrors::new();
            raml_errors.errors
                .push(format!("Invalid yaml: {}", scan_error));
            Err(raml_errors)
        } 
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
