extern crate yaml_rust;

use yaml_rust::scanner::{Scanner, TokenType, Token, Marker};
use std::str::Chars;
use std::fmt::Display;
use std::fmt;

pub use yaml_rust::scanner::ScanError;

pub type RamlResult = Result<Raml, RamlError>;

#[derive(Debug)]
#[derive(PartialEq)]
pub enum Protocol {
    Http,
    Https,
}

#[derive(PartialEq)]
enum TokenTypeDef {
    NoToken,
    StreamStart,
    StreamEnd,
    VersionDirective,
    TagDirective,
    DocumentStart,
    DocumentEnd,
    BlockSequenceStart,
    BlockMappingStart,
    BlockEnd,
    FlowSequenceStart,
    FlowSequenceEnd,
    FlowMappingStart,
    FlowMappingEnd,
    BlockEntry,
    FlowEntry,
    Key,
    Value,
    Alias,
    Anchor,
    Tag,
    Scalar,
}

impl Display for TokenTypeDef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let printable = match *self {
            TokenTypeDef::StreamStart => "Stream-Start",
            TokenTypeDef::BlockMappingStart => "Block-Mapping-Start",
            TokenTypeDef::Scalar => "Scalar",
            TokenTypeDef::Alias => "Alias",
            TokenTypeDef::Anchor => "Ancor",
            TokenTypeDef::BlockEnd => "Block-End",
            TokenTypeDef::BlockEntry => "Block-Entry",
            TokenTypeDef::BlockSequenceStart => "Block-Sequence-Start",
            TokenTypeDef::DocumentEnd => "Document-End",
            TokenTypeDef::DocumentStart => "Document-Start",
            TokenTypeDef::FlowEntry => "Flow-Entry",
            TokenTypeDef::FlowMappingEnd => "Flow-Mapping-End",
            TokenTypeDef::FlowMappingStart => "Flow-Mapping-Start",
            TokenTypeDef::FlowSequenceEnd => "Flow-Sequence-End",
            TokenTypeDef::FlowSequenceStart => "Flow-Sequence-Start",
            TokenTypeDef::Key => "Key",
            TokenTypeDef::NoToken => "No-Token",
            TokenTypeDef::StreamEnd => "Stream-End",
            TokenTypeDef::Tag => "Tag",
            TokenTypeDef::TagDirective => "Tag-Directive",
            TokenTypeDef::Value => "Value",
            TokenTypeDef::VersionDirective => "Value-Directive",
        };
        write!(f, "{}", printable)
    }
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

enum ErrorDef {
    UnexpectedDocumentRoot { field: String },
    UnexpectedEntry {
        expected: TokenTypeDef,
        found: TokenTypeDef,
    },
    UnexpectedEntryMulti {
        expected: Vec<TokenTypeDef>,
        found: TokenTypeDef,
    },
    MissingRamlVersion,
    MissingTitle,
    UnexpectedProtocol,
    ProtocolsMustBeArray,
    MissingProtocols,
}

fn get_error(error: ErrorDef, marker: Option<Marker>) -> RamlError {
    let message = match error {
        ErrorDef::UnexpectedDocumentRoot { field } => {
            format!("Unexpected field found at the document root: {}", field)
        }
        ErrorDef::UnexpectedEntry { expected, found } => {
            format!("Unexpected entry found. Expected {}, Found {}",
                    expected,
                    found)
        }
        ErrorDef::UnexpectedEntryMulti { expected, found } => {
            let expected_display: String = expected.iter()
                .fold("".to_string(), |acc, x| {
                    acc.clone() + if acc.is_empty() { "" } else { "," } + &(format!("{}", x))
                });

            format!("Unexpected entry found. Expected one of {}, Found {}",
                    expected_display,
                    found)
        }
        ErrorDef::MissingRamlVersion => {
            "Document must start with the following RAML comment line: #%RAML 1.0".to_string()
        }
        ErrorDef::MissingTitle => "Error parsing document root. Missing field: title".to_string(),
        ErrorDef::UnexpectedProtocol => {
            "Error parsing document root. Unexpected protocol".to_string()
        }
        ErrorDef::ProtocolsMustBeArray => {
            "Error parsing document root. Protocols must be an array".to_string()
        }
        ErrorDef::MissingProtocols => {
            "Error parsing document root. Protocols must not be empty.".to_string()
        }
    };
    match marker {
        Some(m) => RamlError::with_marker(message.as_str(), m),
        None => RamlError::new(message.as_str()),
    }
}

fn get_token_def(token_type: &TokenType) -> TokenTypeDef {
    match *token_type {
        TokenType::NoToken => TokenTypeDef::NoToken,
        TokenType::StreamStart(_) => TokenTypeDef::StreamStart,
        TokenType::StreamEnd => TokenTypeDef::StreamEnd,
        TokenType::VersionDirective(_, _) => TokenTypeDef::VersionDirective,
        TokenType::TagDirective(_, _) => TokenTypeDef::TagDirective,
        TokenType::DocumentStart => TokenTypeDef::DocumentStart,
        TokenType::DocumentEnd => TokenTypeDef::DocumentEnd,
        TokenType::BlockSequenceStart => TokenTypeDef::BlockSequenceStart,
        TokenType::BlockMappingStart => TokenTypeDef::BlockMappingStart,
        TokenType::BlockEnd => TokenTypeDef::BlockEnd,
        TokenType::FlowSequenceStart => TokenTypeDef::FlowSequenceStart,
        TokenType::FlowSequenceEnd => TokenTypeDef::FlowSequenceEnd,
        TokenType::FlowMappingStart => TokenTypeDef::FlowMappingStart,
        TokenType::FlowMappingEnd => TokenTypeDef::FlowMappingEnd,
        TokenType::BlockEntry => TokenTypeDef::BlockEntry,
        TokenType::FlowEntry => TokenTypeDef::FlowEntry,
        TokenType::Key => TokenTypeDef::Key,
        TokenType::Value => TokenTypeDef::Value,
        TokenType::Alias(_) => TokenTypeDef::Alias,
        TokenType::Anchor(_) => TokenTypeDef::Anchor,
        TokenType::Tag(_, _) => TokenTypeDef::Tag,
        TokenType::Scalar(_, _) => TokenTypeDef::Scalar,

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
                        TokenType::Scalar(_, ref v) if v == "mediaType" => {
                            self.media_types()?;
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

    fn media_types(&mut self) -> Result<(), RamlError> {
        self.expect(TokenTypeDef::Value)?;
        let token = self.next_token();
        match token.1 {
            TokenType::Scalar(_, v) => self.raml.media_types = Some(vec![v]),
            TokenType::FlowSequenceStart => self.raml.media_types = Some(self.get_array_values()?),
            _ => {
                return Err(get_error(ErrorDef::UnexpectedEntryMulti {
                                         expected: vec![TokenTypeDef::Scalar,
                                                        TokenTypeDef::FlowSequenceStart],
                                         found: get_token_def(&token.1),
                                     },
                                     Some(token.0)))
            }
        }

        Ok(())
    }

    fn get_array_values(&mut self) -> Result<Vec<String>, RamlError> {
        let mut values = vec![];
        loop {
            values.push(self.get_scalar_value()?);
            let token = self.next_token();
            match token.1 {
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

    fn protocols(&mut self) -> Result<(), RamlError> {
        let mut protocols: Vec<Protocol> = vec![];
        self.expect(TokenTypeDef::Value)?;

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
                                    return Err(get_error(ErrorDef::UnexpectedProtocol,
                                                         Some(token.0)))
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
                            return Err(get_error(ErrorDef::UnexpectedEntryMulti {
                                                     expected: vec![TokenTypeDef::Scalar,
                                                                    TokenTypeDef::FlowSequenceEnd,
                                                                    TokenTypeDef::FlowEntry],
                                                     found: get_token_def(&token.1),
                                                 },
                                                 Some(token.0)));
                        }
                    }
                }
                _ => return Err(get_error(ErrorDef::ProtocolsMustBeArray, Some(token.0))),
            }
        }

        if protocols.is_empty() {
            Err(get_error(ErrorDef::MissingProtocols, None))
        } else {
            self.raml.protocols = Some(protocols);
            Ok(())
        }
    }

    fn get_value(&mut self) -> Result<String, RamlError> {
        self.expect(TokenTypeDef::Value)?;
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
