extern crate yaml_rust;

use yaml_rust::scanner::{Scanner, TokenType, Token, Marker};
use std::str::Chars;

mod errors;

pub use yaml_rust::scanner::ScanError;

pub type RamlResult = Result<Raml, RamlError>;

#[derive(Debug)]
#[derive(PartialEq)]
pub enum Protocol {
    Http,
    Https,
}

enum ExpectedTokenType {
    StreamStart,
    BlockMappingStart,
}

#[derive(Debug)]
#[derive(PartialEq)]
pub struct Raml {
    title: String,
    version: Option<String>,
    description: Option<String>,
    base_uri: Option<String>,
    protocols: Option<Vec<Protocol>>,
}

impl Raml {
    fn new() -> Raml {
        Raml {
            title: "".to_string(),
            version: None,
            description: None,
            base_uri: None,
            protocols: None,
        }
    }

    pub fn title(&self) -> &str {
        self.title.as_str()
    }

    pub fn version(self) -> Option<String> {
        self.version
    }

    pub fn description(self) -> Option<String> {
        self.description
    }

    pub fn base_uri(self) -> Option<String> {
        self.base_uri
    }

    pub fn protocols(self) -> Option<Vec<Protocol>> {
        self.protocols
    }
}

#[derive(Default)]
#[derive(Debug)]
pub struct RamlError {
    error: String,
}

impl RamlError {
    fn new(error: &str) -> RamlError {
        RamlError { error: error.to_string() }
    }

    fn with_marker(error: &str, marker: Marker) -> RamlError {
        // The marker properties are private, so work around this by constructing a ScanError
        // and use the display format.
        let error = format!("{}", ScanError::new(marker, error));
        RamlError { error: error }
    }

    pub fn error(&self) -> &str {
        self.error.as_str()
    }
}

pub struct RamlParser<'a> {
    scanner: Scanner<Chars<'a>>,
    raml: Raml,
}

impl<'a> RamlParser<'a> {
    pub fn load_from_str(source: &str) -> RamlResult {
        let mut parser = RamlParser {
            scanner: Scanner::new(source.chars()),
            raml: Raml::new(),
        };

        parser.error_if_incorrect_raml_comment(source)?;

        parser.doc_root()?;

        Ok(parser.raml)
    }

    fn error_if_incorrect_raml_comment(&mut self, s: &str) -> Result<(), RamlError> {
        let first_line: &str = s.lines().next().unwrap_or_default().trim();
        if first_line != "#%RAML 1.0" {
            return Err(RamlError::new(errors::ERROR_MISSING_RAML_VERSION));
        }
        Ok(())
    }

    fn doc_root(&mut self) -> Result<(), RamlError> {
        self.expect(ExpectedTokenType::StreamStart)?;
        self.expect(ExpectedTokenType::BlockMappingStart)?;
        loop {
            let token = self.next_token();
            match token.1 {
                TokenType::Key => {
                    let token = self.next_token();
                    match token.1 {
                        TokenType::Scalar(_, ref v) if v == "title" => {
                            self.raml.title = self.get_value()?;
                        }
                        TokenType::Scalar(_, ref v) if v == "version" => {
                            self.raml.version = Some(self.get_value()?);
                        }
                        TokenType::Scalar(_, ref v) if v == "description" => {
                            self.raml.description = Some(self.get_value()?);
                        }
                        TokenType::Scalar(_, ref v) if v == "baseUri" => {
                            self.raml.base_uri = Some(self.get_value()?);
                        }
                        TokenType::Scalar(_, ref v) if v == "protocols" => {
                            self.protocols()?;
                        }
                        TokenType::Scalar(_, ref v) => {
                            return Err(RamlError::with_marker(format!("Unexpected field found \
                                                                       at the document root: {}",
                                                                      v)
                                                                  .as_str(),
                                                              token.0))
                        }
                        _ => return Err(RamlError::with_marker("expected scalar key", token.0)),
                    }
                } 
                TokenType::BlockEnd => {
                    if self.raml.title.is_empty() {
                        return Err(RamlError::new("Error parsing document root. Missing field: \
                                                   title"));
                    } else {
                        break;
                    }
                }
                _ => return Err(RamlError::with_marker("did not find expected <key>", token.0)),
            }
        }
        Ok(())
    }

    fn protocols(&mut self) -> Result<(), RamlError> {
        let mut protocols: Vec<Protocol> = vec![];
        let token = self.next_token();
        match token.1 {
            TokenType::Value => {
                let token = self.next_token();
                loop {
                    match token.1 {
                        TokenType::FlowSequenceStart => {
                            let token = self.next_token();
                            match token.1 {
                                TokenType::Scalar(_, ref v) => {
                                    match v.to_lowercase().as_str() {
                                        "http" => protocols.push(Protocol::Http),
                                        "https" => protocols.push(Protocol::Https),
                                        _ => {
                                            return Err(RamlError::with_marker("Error parsing \
                                                                               document root. \
                                                                               Unexpected \
                                                                               protocol",
                                                                              token.0))
                                        }
                                    }
                                }
                                TokenType::FlowEntry => {
                                    // ignore
                                }
                                TokenType::FlowSequenceEnd => {
                                    break;
                                }
                                _ => {
                                    print!("TOKEN {:?}", token);
                                    return Err(RamlError::new("todo"));
                                }
                            }
                        }
                        _ => {
                            return Err(RamlError::with_marker("Error parsing document root. \
                                                               Protocols must be an array",
                                                              token.0))
                        }
                    }
                }

            }
            _ => return Err(RamlError::with_marker("expected value", token.0)),
        }

        if protocols.is_empty() {
            Err(RamlError::new("Error parsing document root. Protocols must not be empty."))
        } else {
            self.raml.protocols = Some(protocols);
            Ok(())
        }
    }

    fn get_value(&mut self) -> Result<String, RamlError> {
        let token = self.next_token();
        match token.1 {
            TokenType::Value => {
                let token = self.next_token();
                match token.1 {
                    TokenType::Scalar(_, ref v) => Ok(v.clone()),
                    _ => Err(RamlError::with_marker("expected scalar", token.0)),
                }
            }
            _ => Err(RamlError::with_marker("expected value", token.0)),
        }
    }

    fn next_token(&mut self) -> Token {
        // todo error handling
        self.scanner.next().unwrap()
    }

    fn expect(&mut self, expected_token_type: ExpectedTokenType) -> Result<(), RamlError> {
        let token = self.next_token();
        match expected_token_type {
            ExpectedTokenType::StreamStart => {
                if let TokenType::StreamStart(_) = token.1 {
                    Ok(())
                } else {
                    Err(RamlError::with_marker("did not find expected <stream-start>", token.0))
                }
            }
            ExpectedTokenType::BlockMappingStart => {
                if let TokenType::BlockMappingStart = token.1 {
                    Ok(())
                } else {
                    Err(RamlError::with_marker("did not find expected <block-mapping-start>",
                                               token.0))
                }
            }
        }
    }
}
