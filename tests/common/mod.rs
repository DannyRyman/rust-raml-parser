extern crate raml_parser;

use raml_parser::*;

pub fn parse(s: &str) -> RamlResult {
    RamlParser::load_from_str(s)
}

pub fn assert_ok_and_unwrap(result: RamlResult) -> Raml {
    if result.is_err() {
        println!("Unexpected error {:?}", result);
    }
    assert_eq!(result.is_ok(), true);
    result.ok().unwrap()
}

pub fn assert_error_result(result: RamlResult, expected_error: &str) {
    assert_eq!(result.is_err(), true);
    let err = result.err().unwrap();
    assert_eq!(err.error(), expected_error);
}