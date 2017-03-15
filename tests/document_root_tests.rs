extern crate raml_parser;

use raml_parser::parse;
use raml_parser::Raml;
use raml_parser::RamlResult;

#[test]
fn error_on_empty_document() {
    let s = "";
    let result = parse(s);
    assert_error_result(result, &vec!["Attempted to parse an empty document"]);
}

#[test]
fn error_for_missing_version_comment() {
    let s = "title: Some API";
    let result = parse(s);
    assert_error_result(result,
                        &vec!["Document must start with the following RAML comment line: #%RAML \
                               1.0"]);
}

#[test]
fn error_for_invalid_yaml() {
    let s = "%%invalid";
    let result = parse(s);
    assert_error_result(result,
                        &vec!["Invalid yaml: while scanning a directive, could not find \
                               expected directive name at line 1 column 2"]);
}

#[test]
fn error_for_missing_title() {
    let s = "#%RAML 1.0
    version: v1";
    let result = parse(s);
    assert_error_result(result,
                        &vec!["Error parsing document root. Missing field: title"]);
}

#[test]
fn loads_the_title() {
    let s = "#%RAML 1.0
    title: Some API";
    let result = parse(s);
    let raml = assert_ok_and_unwrap(result);
    assert_eq!("Some API", raml.title());
}

#[test]
fn loads_the_version() {
    let s = "#%RAML 1.0
    title: Some API
    version: v1";
    let result = parse(s);
    let raml = assert_ok_and_unwrap(result);
    assert_eq!("v1", raml.version().unwrap());
}

fn assert_ok_and_unwrap(result: RamlResult) -> Raml {
    assert_eq!(result.is_ok(), true);
    result.ok().unwrap()
}

fn assert_error_result(result: RamlResult, expected_errors: &Vec<&str>) {
    assert_eq!(result.is_err(), true);
    let err = result.err().unwrap();
    let errors = err.errors();
    assert_eq!(errors, expected_errors);
}