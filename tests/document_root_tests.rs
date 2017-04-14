extern crate raml_parser;

use raml_parser::*;

fn parse(s: &str) -> RamlResult {
    RamlParser::load_from_str(s)
}

#[test]
fn error_for_missing_version_comment() {
    let s = "title: Some API";
    let result = parse(s);
    assert_error_result(result,
                        "Document must start with the following RAML comment line: #%RAML 1.0");
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

// todo baseUriParameters

#[test]
fn loads_the_protocols_ignoring_casing() {
    let s = "#%RAML 1.0
    title: Some API
    protocols: [http, HTTPS]";
    let result = parse(s);
    let raml = assert_ok_and_unwrap(result);
    assert_eq!(vec![Protocol::Http, Protocol::Https],
               raml.protocols().unwrap());
}

#[test]
fn error_for_empty_protocols() {
    let s = "#%RAML 1.0
title: Some API
protocols: []";
    let result = parse(s);
    assert_error_result(result,
                        "Error parsing document root. Protocols must not be empty");
}

#[test]
fn error_when_protocols_is_not_array() {
    let s = "#%RAML 1.0
title: Some API
protocols: http";
    let result = parse(s);
    assert_error_result(result,
                        "Unexpected entry found. Expected Flow-Sequence-Start, Found Scalar at \
                         line 3 column 12");
}

#[test]
fn error_for_unexpected_protocol() {
    let s = "#%RAML 1.0
title: Some API
protocols: [Invalid]";
    let result = parse(s);
    assert_error_result(result,
                        "Error parsing document root. Unexpected protocol at line 3 column 13");
}

#[test]
fn media_type_single_value() {
    let s = "#%RAML 1.0
title: Some API
mediaType: application/json";
    let result = parse(s);
    let raml = assert_ok_and_unwrap(result);
    assert_eq!(vec!["application/json"], raml.media_types().unwrap());
}

#[test]
fn media_type_multiple_values() {
    let s = "#%RAML 1.0
title: Some API
mediaType: [application/json, application/xml]";
    let result = parse(s);
    let raml = assert_ok_and_unwrap(result);
    assert_eq!(vec!["application/json", "application/xml"],
               raml.media_types().unwrap());
}

#[test]
fn no_media_type_must_result_in_none() {
    let s = "#%RAML 1.0
title: Some API";
    let result = parse(s);
    let raml = assert_ok_and_unwrap(result);
    assert_eq!(None, raml.media_types());
}

#[test]
fn loads_the_documentation() {
    let s = "#%RAML 1.0
title: Some API
documentation:
 - title: Doc Title
   content: Doc Content";

    let result = parse(s);
    let raml = assert_ok_and_unwrap(result);
    assert_eq!(vec![RamlDocumentation::new("Doc Title".to_string(), "Doc Content".to_string())],
               raml.documentation().unwrap());
}

#[test]
fn loads_multiple_documents() {
    let s = "#%RAML 1.0
title: Some API
documentation:
 - title: Doc Title
   content: Doc Content
 - title: Doc Title2
   content: Doc Content2";

    let expected = vec![RamlDocumentation::new("Doc Title".to_string(), "Doc Content".to_string()),
                        RamlDocumentation::new("Doc Title2".to_string(),
                                               "Doc Content2".to_string())];

    let result = parse(s);
    let raml = assert_ok_and_unwrap(result);
    assert_eq!(expected, raml.documentation().unwrap());
}

#[test]
fn error_for_empty_documentation() {
    let s = "#%RAML 1.0
title: Some API
documentation:";
    let result = parse(s);
    assert_error_result(result,
                        "Unexpected entry found. Expected Block-Sequence-Start, Found Block-End \
                         at line 4 column 1")
}

#[test]
fn documentation_content_over_multiple_lines() {
    let s = "#%RAML 1.0
title: Some API
documentation:
 - title: Doc Title
   content: Here is some content
            over multiple lines";

    let result = parse(s);
    let raml = assert_ok_and_unwrap(result);
    assert_eq!(vec![RamlDocumentation::new("Doc Title".to_string(),
                                           "Here is some content over multiple lines"
                                               .to_string())],
               raml.documentation().unwrap());
}

#[test]
fn error_for_unexpected_documentation_key() {
    let s = "#%RAML 1.0
title: Some API
documentation:
 - title1: Doc title
   content: Doc Content";
    let result = parse(s);
    assert_error_result(result,
                        "Unexpected field found at the documentation: title1 at line 4 column 4")
}

#[test]
fn error_missing_documentation_title() {
    let s = "#%RAML 1.0
title: Some API
documentation:
 - content: Doc Content";
    let result = parse(s);
    assert_error_result(result, "Error parsing documentation. Missing field: title")
}

// Missing title
// Missing content

#[test]
fn error_for_unknown_field() {
    let s = "#%RAML 1.0
title: Some API
unknown: field";
    let result = parse(s);
    assert_error_result(result,
                        "Unexpected field found at the document root: unknown at line 3 column 1");
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