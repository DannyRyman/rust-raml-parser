extern crate raml_parser;

// use raml_parser::parse;
// use raml_parser::Raml;
// use raml_parser::RamlResult;

use raml_parser::RamlParser;
use raml_parser::Raml;
use raml_parser::RamlResult;

fn parse(s: &str) -> RamlResult {
    RamlParser::load_from_str(s)
}


#[test]
fn error_for_missing_version_comment() {
    let s = "title: Some API";
    let result = parse(s);
    assert_error_result(result, "Document must start with the following RAML comment line: #%RAML 1.0");
}

#[test]
fn error_for_missing_title() {
    let s = "#%RAML 1.0
    version: v1";
    let result = parse(s);
    assert_error_result(result, "Error parsing document root. Missing field: title");
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

#[test]
fn loads_the_description() {
    let s = "#%RAML 1.0
    title: Some API
    description: Sample description";
    let result = parse(s);
    let raml = assert_ok_and_unwrap(result);
    assert_eq!("Sample description", raml.description().unwrap());
}

#[test]
fn loads_the_base_uri() {
    let s = "#%RAML 1.0
    title: Some API
    baseUri: https://some.api.com/{version}";
    let result = parse(s);
    let raml = assert_ok_and_unwrap(result);
    assert_eq!("https://some.api.com/{version}", raml.base_uri().unwrap());
}

#[test]
fn error_for_unknown_field() {
    let s = "#%RAML 1.0
title: Some API
unknown: field";
    let result = parse(s);
    assert_error_result(result, "Unexpected field found at the document root: unknown at line 3 column 1");
}

fn assert_ok_and_unwrap(result: RamlResult) -> Raml {
    if result.is_err() {
        println!("Unexpected error {:?}", result);
    }
    assert_eq!(result.is_ok(), true);
    result.ok().unwrap()
}

fn assert_error_result(result: RamlResult, expected_error: &str) {
    assert_eq!(result.is_err(), true);
    let err = result.err().unwrap();
    assert_eq!(err.error(), expected_error);
}