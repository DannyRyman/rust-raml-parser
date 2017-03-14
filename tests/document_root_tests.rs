extern crate raml_parser;

use raml_parser::parse;

#[test]
fn error_on_empty_document() {
    let s = "";
    let result = parse(s);
    assert_eq!(result.is_err(), true);
    let err = result.err().unwrap();
    let errors = err.errors();
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0], "Attempted to parse an empty document");
}

#[test]
fn error_for_missing_version_comment() {
    let s = "title: Some API";
    let result = parse(s);
    assert_eq!(result.is_err(), true);
    let err = result.err().unwrap();
    let errors = err.errors();
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0],
               "Document must start with the following RAML comment line: #%RAML 1.0");
}

#[test]
fn error_for_invalid_yaml() {
    let s = "%%invalid";
    let result = parse(s);
    assert_eq!(result.is_err(), true);
    let err = result.err().unwrap();
    let errors = err.errors();
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
    assert_eq!("Some API", raml.title());
}

#[test]
fn error_for_missing_title() {
    let s = "#%RAML 1.0
    version: v1";
    let result = parse(s);
    assert_eq!(result.is_err(), true);
    let err = result.err().unwrap();
    let errors = err.errors();
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
    assert_eq!("Some API", raml.title());
    assert_eq!("v1", raml.version().unwrap());
}
