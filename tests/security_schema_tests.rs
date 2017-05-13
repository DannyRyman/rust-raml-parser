#![cfg_attr(test, allow(dead_code))]

extern crate raml_parser;

use raml_parser::*;

mod common;

use common::*;

#[test]
fn minimum_security_scheme() {
    let s = "#%RAML 1.0
title: Some API
securitySchemes:
  oauth_2_0:
    type: OAuth 2.0";

    let result = parse(s);
    let raml = assert_ok_and_unwrap(result);
    let security_schemes = raml.security_schemes().unwrap();
    assert_eq!(SecuritySchemeType::OAuth2,
               security_schemes.get("oauth_2_0").unwrap().security_type);
}

#[test]
fn x_other_security_type() {
    let s = "#%RAML 1.0
title: Some API
securitySchemes:
  custom:
    type: x-custom";

    let result = parse(s);
    let raml = assert_ok_and_unwrap(result);
    let security_schemes = raml.security_schemes().unwrap();
    assert_eq!(SecuritySchemeType::XOther("x-custom".to_string()),
               security_schemes.get("custom").unwrap().security_type);
}

#[test]
fn valid_display_name_and_description() {
    let s = "#%RAML 1.0
title: Some API
securitySchemes:
  oauth_2_0:
    type: OAuth 2.0
    displayName: sample display name
    description: |
      sample description";

    let result = parse(s);
    let raml = assert_ok_and_unwrap(result);
    let security_schemes = raml.security_schemes().unwrap();
    assert_eq!(Some("sample display name".to_string()),
               security_schemes.get("oauth_2_0").unwrap().display_name);
    assert_eq!(Some("sample description".to_string()),
               security_schemes.get("oauth_2_0").unwrap().description);
}

#[test]
fn if_security_scheme_is_specified_it_must_have_entries() {
    let s = "#%RAML 1.0
title: Some API
securitySchemes:";
    let result = parse(s);
    assert_error_result(result,
                        "Unexpected entry found. Expected Block-Mapping-Start, Found Block-End \
                         at line 4 column 1")
}

#[test]
fn error_if_missing_type() {
    let s = "#%RAML 1.0
title: Some API
securitySchemes:
  oauth_2_0:
    displayName: sample display name";
    let result = parse(s);
    assert_error_result(result, "Error parsing security scheme. Missing field: type")
}

#[test]
fn if_described_by_is_specified_it_must_have_entries() {
    let s = "#%RAML 1.0
title: Some API
securitySchemes:
  oauth_2_0:
    describedBy:";
    let result = parse(s);
    assert_error_result(result,
                        "Unexpected entry found. Expected Block-Mapping-Start, Found Block-End \
                         at line 6 column 1")
}

#[test]
fn valid_described_by_headers() {}
