use yaml_rust::scanner::{Scanner, TokenType, Token, Marker};
use std::str::{Chars, FromStr};
use error_definitions::{ErrorDef, RamlError, get_error, HierarchyLevel};
use token_type_definitions::{TokenTypeDef, get_token_def};
use std::collections::HashMap;

pub type RamlResult = Result<Raml, RamlError>;

#[derive(Debug)]
#[derive(PartialEq)]
pub enum Protocol {
    Http,
    Https,
}

pub type Protocols = Vec<Protocol>;

#[derive(Debug)]
#[derive(PartialEq)]
pub struct Raml {
    title: String,
    version: Option<String>,
    description: Option<String>,
    base_uri: Option<String>,
    protocols: Option<Vec<Protocol>>,
    media_types: Option<Vec<String>>,
    documentation: Option<Vec<RamlDocumentation>>,
    security_schemes: Option<SecuritySchemes>,
}

struct KeyValue {
    key: String,
    value: String,
}

#[derive(Debug)]
#[derive(PartialEq)]
pub struct RamlDocumentation {
    title: String,
    content: String,
}

impl RamlDocumentation {
    pub fn new(title: String, content: String) -> RamlDocumentation {
        RamlDocumentation {
            title: title,
            content: content,
        }
    }
}

type FlowSequenceEntries = Vec<FlowSequenceEntry>;

pub type SecuritySchemes = HashMap<String, SecurityScheme>;

#[derive(Debug)]
#[derive(PartialEq)]
pub enum SecuritySchemeType {
    OAuth1,
    OAuth2,
    BasicAuthentication,
    DigestAuthentication,
    PassThrough,
    XOther(String),
}

impl FromStr for SecuritySchemeType {
    type Err = RamlError;

    fn from_str(s: &str) -> Result<SecuritySchemeType, RamlError> {
        match s.to_lowercase().as_str() {
            "oauth 1.0" => Ok(SecuritySchemeType::OAuth1),
            "oauth 2.0" => Ok(SecuritySchemeType::OAuth2),
            "basic authentication" => Ok(SecuritySchemeType::BasicAuthentication),
            "digest authentication" => Ok(SecuritySchemeType::DigestAuthentication),
            "pass through" => Ok(SecuritySchemeType::PassThrough),
            s if s.starts_with("x-") => Ok(SecuritySchemeType::XOther(s.to_string())),
            _ => Err(get_error(ErrorDef::InvalidSecuritySchemeType, None)),
        }
    }
}

#[derive(PartialEq)]
#[derive(Debug)]
pub struct SecurityScheme {
    pub security_type: SecuritySchemeType,
}

type BlockSequenceEntries = HashMap<String, BlockSequenceEntry>;

struct BlockSequenceEntry {
    value: String,
    marker: Marker,
}

type VectorOfBlockSequenceEntries = Vec<BlockSequenceEntries>;

struct FlowSequenceEntry {
    value: String,
    marker: Marker,
}

impl Raml {
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

    pub fn documentation(self) -> Option<Vec<RamlDocumentation>> {
        self.documentation
    }

    pub fn security_schemes(self) -> Option<SecuritySchemes> {
        self.security_schemes
    }
}

struct ForwardCursor<'a> {
    scanner: Scanner<Chars<'a>>,
}

impl<'a> ForwardCursor<'a> {
    fn new(source: &str) -> ForwardCursor {
        ForwardCursor { scanner: Scanner::new(source.chars()) }
    }

    fn next_token(&mut self) -> Token {
        // todo error handling
        self.scanner.next().unwrap()
        // let token_def = get_token_def(&token.1);
        // println!("Token {}", token_def);
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

fn print_tokens(source: &str) {
    let mut cursor = ForwardCursor::new(source);
    loop {
        let token = cursor.next_token();
        println!("Token {:?}", token.1);
        if let TokenType::StreamEnd = token.1 {
            break;
        }
    }
}

fn parse_raml_string(source: &str) -> RamlResult {
    error_if_incorrect_raml_comment(source)?;
    let mut cursor = ForwardCursor::new(source);
    parse_root(&mut cursor)
}

fn get_scalar_value(cursor: &mut ForwardCursor) -> Result<String, RamlError> {
    let token = cursor.next_token();
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

fn get_flow_sequence(cursor: &mut ForwardCursor) -> Result<FlowSequenceEntries, RamlError> {
    let mut values = vec![];
    loop {
        let token = cursor.next_token();
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

fn get_multiple_values(cursor: &mut ForwardCursor) -> Result<FlowSequenceEntries, RamlError> {
    cursor.expect(TokenTypeDef::Value)?;
    cursor.expect(TokenTypeDef::FlowSequenceStart)?;
    get_flow_sequence(cursor)
}

fn get_multiple_sets_of_values(cursor: &mut ForwardCursor)
                               -> Result<VectorOfBlockSequenceEntries, RamlError> {
    cursor.expect(TokenTypeDef::Value)?;
    cursor.expect(TokenTypeDef::BlockSequenceStart)?;
    get_block_sequences(cursor)
}

fn get_single_value(cursor: &mut ForwardCursor) -> Result<String, RamlError> {
    cursor.expect(TokenTypeDef::Value)?;
    get_scalar_value(cursor)
}

fn get_block_sequences(cursor: &mut ForwardCursor)
                       -> Result<VectorOfBlockSequenceEntries, RamlError> {
    let mut result: VectorOfBlockSequenceEntries = Vec::new();
    loop {
        let token = cursor.next_token();
        match token.1 {
            TokenType::BlockEntry => {
                let block_sequence = get_block_sequence(cursor)?;
                result.push(block_sequence);
            }
            TokenType::BlockEnd => {
                break;
            }
            _ => {
                return Err(get_error(ErrorDef::UnexpectedEntryMulti {
                                         expected: vec![TokenTypeDef::BlockEntry,
                                                        TokenTypeDef::BlockEnd],
                                         found: get_token_def(&token.1),
                                     },
                                     Some(token.0)))
            }
        }
    }
    Ok(result)
}

fn get_key_value(cursor: &mut ForwardCursor) -> Result<KeyValue, RamlError> {
    let key = get_scalar_value(cursor)?;
    cursor.expect(TokenTypeDef::Value)?;
    let value = get_scalar_value(cursor)?;
    Ok(KeyValue {
        key: key,
        value: value,
    })
}

fn get_block_sequence(cursor: &mut ForwardCursor) -> Result<BlockSequenceEntries, RamlError> {
    let mut result: BlockSequenceEntries = HashMap::new();
    cursor.expect(TokenTypeDef::BlockMappingStart)?;
    loop {
        let token = cursor.next_token();
        match token.1 {
            TokenType::Key => {
                let key_value = get_key_value(cursor)?;
                result.insert(key_value.key,
                              BlockSequenceEntry {
                                  value: key_value.value,
                                  marker: token.0,
                              });
            }
            TokenType::BlockEnd => {
                break;
            }
            _ => {
                return Err(get_error(ErrorDef::UnexpectedEntryMulti {
                                         expected: vec![TokenTypeDef::Key, TokenTypeDef::BlockEnd],
                                         found: get_token_def(&token.1),
                                     },
                                     Some(token.0)))
            }
        }
    }
    Ok(result)
}

fn get_single_or_multiple_values(cursor: &mut ForwardCursor)
                                 -> Result<FlowSequenceEntries, RamlError> {
    cursor.expect(TokenTypeDef::Value)?;

    let token = cursor.next_token();
    match token.1 {
        TokenType::Scalar(_, v) => {
            Ok(vec![FlowSequenceEntry {
                        value: v,
                        marker: token.0,
                    }])
        }
        TokenType::FlowSequenceStart => get_flow_sequence(cursor),
        _ => {
            Err(get_error(ErrorDef::UnexpectedEntryMulti {
                              expected: vec![TokenTypeDef::Scalar, TokenTypeDef::FlowSequenceStart],
                              found: get_token_def(&token.1),
                          },
                          Some(token.0)))
        }
    }
}

fn get_protocols(cursor: &mut ForwardCursor) -> Result<Protocols, RamlError> {
    let protocols = get_multiple_values(cursor)?;
    if protocols.is_empty() {
        return Err(get_error(ErrorDef::MissingProtocols, None));
    }
    let protocols: Result<Protocols, RamlError> = protocols.iter()
        .map(|p| match p.value.to_lowercase().as_str() {
            "http" => Ok(Protocol::Http),
            "https" => Ok(Protocol::Https),
            _ => Err(get_error(ErrorDef::UnexpectedProtocol, Some(p.marker))),
        })
        .collect();

    Ok(protocols?)
}

pub type MediaTypes = Vec<String>;

fn get_media_types(cursor: &mut ForwardCursor) -> Result<MediaTypes, RamlError> {
    let media_types = get_single_or_multiple_values(cursor)
        ?
        .iter()
        .map(|e| e.value.clone())
        .collect();
    Ok(media_types)
}

pub type RamlDocumentationEntries = Vec<RamlDocumentation>;

fn get_documentation(cursor: &mut ForwardCursor) -> Result<RamlDocumentationEntries, RamlError> {
    let documentation_result: Result<RamlDocumentationEntries, RamlError> =
        get_multiple_sets_of_values(cursor)
            ?
            .iter()
            .map(|s| {
                let mut title: Option<String> = None;
                let mut content: Option<String> = None;
                for (key, entry) in s {
                    println!("***** {}: {}", key, entry.value);
                    if key == "title" {
                        title = Some(entry.value.clone())
                    } else if key == "content" {
                        content = Some(entry.value.clone())
                    } else {
                        println!("unexpected key: {}", key);
                        return Err(get_error(ErrorDef::UnexpectedKeyRoot {
                                                 field: key.to_string(),
                                                 level: HierarchyLevel::Documentation,
                                             },
                                             Some(entry.marker)));
                    }
                }
                if title.is_none() {
                    return Err(get_error(ErrorDef::MissingField {
                                             field: "title".to_string(),
                                             level: HierarchyLevel::Documentation,
                                         },
                                         None));
                }
                Ok(RamlDocumentation {
                    title: title.unwrap(),
                    content: content.unwrap(),
                })
            })
            .collect();

    Ok(documentation_result?)
}

fn get_security_schemes(cursor: &mut ForwardCursor) -> Result<SecuritySchemes, RamlError> {
    let mut result: SecuritySchemes = HashMap::new();
    cursor.expect(TokenTypeDef::Value)?;
    cursor.expect(TokenTypeDef::BlockMappingStart)?;

    loop {
        let token = cursor.next_token();
        match token.1 {
            TokenType::Key => {
                let token = cursor.next_token();
                match token.1 {
                    TokenType::Scalar(_, v) => {
                        result.insert(v, get_security_scheme(cursor)?);
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
                break;
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


    // let entries = self.get_multiple_sets_of_values()?;

    // result.insert("oauth_2_0".to_string(),
    //               SecurityScheme { security_type: SecuritySchemeType::OAuth2 });

    Ok(result)
}

fn get_security_scheme(cursor: &mut ForwardCursor) -> Result<SecurityScheme, RamlError> {
    let mut security_type: Option<SecuritySchemeType> = None;
    cursor.expect(TokenTypeDef::Value)?;
    cursor.expect(TokenTypeDef::BlockMappingStart)?;
    loop {
        let token = cursor.next_token();
        match token.1 {
            TokenType::Key => {
                let token = cursor.next_token();
                match token.1 {
                    TokenType::Scalar(_, ref v) if v == "type" => {
                        let security_type_str = get_single_value(cursor)?;
                        security_type = Some(security_type_str.parse::<SecuritySchemeType>()?);
                    }
                    TokenType::Scalar(_, v) => {
                        return Err(get_error(ErrorDef::UnexpectedKeyRoot {
                                                 field: v,
                                                 level: HierarchyLevel::DocumentRoot,
                                             },
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
                break;
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

    Ok(SecurityScheme { security_type: security_type.unwrap() })
}

fn parse_root(cursor: &mut ForwardCursor) -> RamlResult {
    cursor.expect(TokenTypeDef::StreamStart)?;
    cursor.expect(TokenTypeDef::BlockMappingStart)?;
    let mut title: Option<String> = None;
    let mut version: Option<String> = None;
    let mut description: Option<String> = None;
    let mut base_uri: Option<String> = None;
    let mut protocols: Option<Protocols> = None;
    let mut media_types: Option<MediaTypes> = None;
    let mut documentation: Option<RamlDocumentationEntries> = None;
    let mut security_schemes: Option<SecuritySchemes> = None;
    loop {
        let token = cursor.next_token();
        match token.1 {
            TokenType::Key => {
                let token = cursor.next_token();
                match token.1 {
                    TokenType::Scalar(_, ref v) if v == "title" => {
                        title = Some(get_single_value(cursor)?);
                    }
                    TokenType::Scalar(_, ref v) if v == "version" => {
                        version = Some(get_single_value(cursor)?);
                    }
                    TokenType::Scalar(_, ref v) if v == "description" => {
                        description = Some(get_single_value(cursor)?);
                    }
                    TokenType::Scalar(_, ref v) if v == "baseUri" => {
                        base_uri = Some(get_single_value(cursor)?);
                    }
                    TokenType::Scalar(_, ref v) if v == "protocols" => {
                        protocols = Some(get_protocols(cursor)?);
                    }
                    TokenType::Scalar(_, ref v) if v == "mediaType" => {
                        media_types = Some(get_media_types(cursor)?);
                    }
                    TokenType::Scalar(_, ref v) if v == "documentation" => {
                        documentation = Some(get_documentation(cursor)?);
                    }
                    TokenType::Scalar(_, ref v) if v == "securitySchemes" => {
                        security_schemes = Some(get_security_schemes(cursor)?);
                    }
                    TokenType::Scalar(_, v) => {
                        return Err(get_error(ErrorDef::UnexpectedKeyRoot {
                                                 field: v,
                                                 level: HierarchyLevel::DocumentRoot,
                                             },
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
                if title.is_none() {
                    return Err(get_error(ErrorDef::MissingField {
                                             field: "title".to_string(),
                                             level: HierarchyLevel::DocumentRoot,
                                         },
                                         None));
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
    Ok(Raml {
        title: title.unwrap(),
        version: version,
        description: description,
        base_uri: base_uri,
        protocols: protocols,
        media_types: media_types,
        documentation: documentation,
        security_schemes: security_schemes,
    })
}

fn error_if_incorrect_raml_comment(s: &str) -> Result<(), RamlError> {
    let first_line: &str = s.lines().next().unwrap_or_default().trim();
    if first_line != "#%RAML 1.0" {
        return Err(get_error(ErrorDef::MissingRamlVersion, None));
    }
    Ok(())
}

pub struct RamlParser {}

impl RamlParser {
    pub fn debug(source: &str) {
        print_tokens(source);
    }

    pub fn load_from_str(source: &str) -> RamlResult {
        parse_raml_string(source)
    }
}
