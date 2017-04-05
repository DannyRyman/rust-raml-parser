use std::fmt::Display;
use std::fmt;
use yaml_rust::scanner::TokenType;

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