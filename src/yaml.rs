use std::collections::HashMap;
use yaml_rust::scanner::{TokenType, Marker, Scanner, Token};
use error_definitions::RamlError;
use error_definitions::{get_error, ErrorDef};
use std::str::Chars;
use std::fmt::Display;
use std::fmt;

pub type BlockSequenceEntries = HashMap<String, BlockSequenceEntry>;

pub struct BlockSequenceEntry {
    pub value: String,
    pub marker: Marker,
}

pub type VectorOfBlockSequenceEntries = Vec<BlockSequenceEntries>;

pub struct FlowSequenceEntry {
    pub value: String,
    pub marker: Marker,
}

pub struct KeyValue {
    pub key: String,
    pub value: String,
}

pub type FlowSequenceEntries = Vec<FlowSequenceEntry>;

#[derive(Debug)]
#[derive(PartialEq)]
pub enum TokenTypeDef {
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

pub fn get_token_def(token_type: &TokenType) -> TokenTypeDef {
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

pub fn get_scalar_value(cursor: &mut ForwardCursor) -> Result<String, RamlError> {
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

pub fn get_flow_sequence(cursor: &mut ForwardCursor) -> Result<FlowSequenceEntries, RamlError> {
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

pub fn get_multiple_values(cursor: &mut ForwardCursor) -> Result<FlowSequenceEntries, RamlError> {
    cursor.expect(TokenTypeDef::Value)?;
    cursor.expect(TokenTypeDef::FlowSequenceStart)?;
    get_flow_sequence(cursor)
}

pub fn get_multiple_sets_of_values(cursor: &mut ForwardCursor)
                                   -> Result<VectorOfBlockSequenceEntries, RamlError> {
    cursor.expect(TokenTypeDef::Value)?;
    cursor.expect(TokenTypeDef::BlockSequenceStart)?;
    get_block_sequences(cursor)
}

pub fn get_single_value(cursor: &mut ForwardCursor) -> Result<String, RamlError> {
    cursor.expect(TokenTypeDef::Value)?;
    get_scalar_value(cursor)
}

pub fn get_block_sequences(cursor: &mut ForwardCursor)
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

pub fn get_key_value(cursor: &mut ForwardCursor) -> Result<KeyValue, RamlError> {
    let key = get_scalar_value(cursor)?;
    cursor.expect(TokenTypeDef::Value)?;
    let value = get_scalar_value(cursor)?;
    Ok(KeyValue {
        key: key,
        value: value,
    })
}

pub fn get_block_sequence(cursor: &mut ForwardCursor) -> Result<BlockSequenceEntries, RamlError> {
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

pub fn get_single_or_multiple_values(cursor: &mut ForwardCursor)
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

pub struct ForwardCursor<'a> {
    scanner: Scanner<Chars<'a>>,
}

impl<'a> ForwardCursor<'a> {
    pub fn new(source: &str) -> ForwardCursor {
        ForwardCursor { scanner: Scanner::new(source.chars()) }
    }

    pub fn next_token(&mut self) -> Token {
        // todo error handling
        self.scanner.next().unwrap()
        // let token_def = get_token_def(&token.1);
        // println!("Token {}", token_def);
    }

    pub fn expect(&mut self, expected_token_type: TokenTypeDef) -> Result<(), RamlError> {
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