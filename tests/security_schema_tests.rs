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
