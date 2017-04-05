use yaml_rust::scanner::{Scanner, TokenType, Token, Marker};
use std::str::Chars;
use error_definitions::{ErrorDef, RamlError, get_error};
use token_type_definitions::{TokenTypeDef, get_token_def};

pub type RamlResult = Result<Raml, RamlError>;

#[derive(Debug)]
#[derive(PartialEq)]
pub enum Protocol {
    Http,
    Https,
}

#[derive(Debug)]
#[derive(PartialEq)]
pub struct Raml {
    title: String,
    version: Option<String>,
    description: Option<String>,
    base_uri: Option<String>,
    protocols: Option<Vec<Protocol>>,
    media_types: Option<Vec<String>>,
}

type FlowSequenceEntries = Vec<FlowSequenceEntry>;

struct FlowSequenceEntry {
    value: String,
    marker: Marker,
}

impl Raml {
    fn new() -> Raml {
        Raml {
            title: "".to_string(),
            version: None,
            description: None,
            base_uri: None,
            protocols: None,
            media_types: None,
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

    pub fn media_types(self) -> Option<Vec<String>> {
        self.media_types
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
            return Err(get_error(ErrorDef::MissingRamlVersion, None));
        }
        Ok(())
    }

    fn doc_root(&mut self) -> Result<(), RamlError> {
        self.expect(TokenTypeDef::StreamStart)?;
        self.expect(TokenTypeDef::BlockMappingStart)?;
        loop {
            let token = self.next_token();
            match token.1 {
                TokenType::Key => {
                    let token = self.next_token();
                    match token.1 {
                        TokenType::Scalar(_, ref v) if v == "title" => {
                            self.raml.title = self.get_single_value()?;
                        }
                        TokenType::Scalar(_, ref v) if v == "version" => {
                            self.raml.version = Some(self.get_single_value()?);
                        }
                        TokenType::Scalar(_, ref v) if v == "description" => {
                            self.raml.description = Some(self.get_single_value()?);
                        }
                        TokenType::Scalar(_, ref v) if v == "baseUri" => {
                            self.raml.base_uri = Some(self.get_single_value()?);
                        }
                        TokenType::Scalar(_, ref v) if v == "protocols" => {
                            let protocols = self.get_multiple_values()?;
                            if protocols.is_empty() {
                                return Err(get_error(ErrorDef::MissingProtocols, None));
                            }
                            let protocols: Result<Vec<Protocol>, RamlError> = protocols.iter()
                                .map(|p| match p.value.to_lowercase().as_str() {
                                    "http" => Ok(Protocol::Http),
                                    "https" => Ok(Protocol::Https),
                                    _ => {
                                        Err(get_error(ErrorDef::UnexpectedProtocol, Some(p.marker)))
                                    }
                                })
                                .collect();
                            self.raml.protocols = Some(protocols?);
                        }
                        TokenType::Scalar(_, ref v) if v == "mediaType" => {
                            self.raml.media_types = Some(self.get_single_or_multiple_values()?
                                .iter()
                                .map(|e| e.value.clone())
                                .collect());
                        }
                        TokenType::Scalar(_, v) => {
                            return Err(get_error(ErrorDef::UnexpectedDocumentRoot { field: v },
                                                 Some(token.0)));
                        }
                        _ => {
                            return Err(get_error(ErrorDef::UnexpectedEntry {
                                                     expected: TokenTypeDef::Scalar,
                                                     found: get_token_def(&token.1),
                                                 },
                                                 Some(token.0)))
                        }
                    }
                } 
                TokenType::BlockEnd => {
                    if self.raml.title.is_empty() {
                        return Err(get_error(ErrorDef::MissingTitle, None));
                    } else {
                        break;
                    }
                }
                _ => {
                    return Err(get_error(ErrorDef::UnexpectedEntry {
                                             expected: TokenTypeDef::Key,
                                             found: get_token_def(&token.1),
                                         },
                                         Some(token.0)))
                }
            }
        }
        Ok(())
    }

    fn get_single_or_multiple_values(&mut self) -> Result<FlowSequenceEntries, RamlError> {
        self.expect(TokenTypeDef::Value)?;

        let token = self.next_token();
        match token.1 {
            TokenType::Scalar(_, v) => {
                Ok(vec![FlowSequenceEntry {
                            value: v,
                            marker: token.0,
                        }])
            }
            TokenType::FlowSequenceStart => self.get_flow_sequence(),
            _ => {
                Err(get_error(ErrorDef::UnexpectedEntryMulti {
                                  expected: vec![TokenTypeDef::Scalar,
                                                 TokenTypeDef::FlowSequenceStart],
                                  found: get_token_def(&token.1),
                              },
                              Some(token.0)))
            }
        }
    }

    fn get_multiple_values(&mut self) -> Result<FlowSequenceEntries, RamlError> {
        self.expect(TokenTypeDef::Value)?;
        self.expect(TokenTypeDef::FlowSequenceStart)?;
        self.get_flow_sequence()
    }

    fn get_flow_sequence(&mut self) -> Result<FlowSequenceEntries, RamlError> {
        let mut values = vec![];
        loop {
            let token = self.next_token();
            match token.1 {
                TokenType::Scalar(_, s) => {
                    values.push(FlowSequenceEntry {
                        value: s,
                        marker: token.0,
                    });
                }
                TokenType::FlowEntry => {
                    // ignore
                }
                TokenType::FlowSequenceEnd => break,
                _ => {
                    return Err(get_error(ErrorDef::UnexpectedEntryMulti {
                                             expected: vec![TokenTypeDef::FlowEntry,
                                                            TokenTypeDef::FlowSequenceEnd],
                                             found: get_token_def(&token.1),
                                         },
                                         Some(token.0)))
                }
            }
        }

        Ok(values)
    }

    fn get_scalar_value(&mut self) -> Result<String, RamlError> {
        let token = self.next_token();
        match token.1 {
            TokenType::Scalar(_, ref v) => Ok(v.clone()),
            _ => {
                Err(get_error(ErrorDef::UnexpectedEntry {
                                  expected: TokenTypeDef::Scalar,
                                  found: get_token_def(&token.1),
                              },
                              Some(token.0)))
            }
        }
    }

    fn get_single_value(&mut self) -> Result<String, RamlError> {
        self.expect(TokenTypeDef::Value)?;
        self.get_scalar_value()
    }

    fn next_token(&mut self) -> Token {
        // todo error handling
        self.scanner.next().unwrap()
    }

    fn expect(&mut self, expected_token_type: TokenTypeDef) -> Result<(), RamlError> {
        let token = self.next_token();
        let found_token_type = get_token_def(&token.1);
        if found_token_type == expected_token_type {
            Ok(())
        } else {
            Err(get_error(ErrorDef::UnexpectedEntry {
                              expected: expected_token_type,
                              found: found_token_type,
                          },
                          Some(token.0)))
        }
    }
}
