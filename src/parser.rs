use yaml_rust::scanner::TokenType;
use error_definitions::{ErrorDef, RamlError, get_error, HierarchyLevel};
use std::collections::HashMap;
use yaml::*;
use std::str::FromStr;

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
    protocols: Option<Protocols>,
    media_types: Option<MediaTypes>,
    documentation: Option<RamlDocumentationEntries>,
    security_schemes: Option<SecuritySchemes>,
}

#[derive(Debug)]
#[derive(PartialEq)]
pub struct RamlDocumentation {
    title: String,
    content: String,
}

pub type RamlDocumentationEntries = Vec<RamlDocumentation>;

impl RamlDocumentation {
    pub fn new(title: String, content: String) -> RamlDocumentation {
        RamlDocumentation {
            title: title,
            content: content,
        }
    }

    pub fn title(&self) -> &str {
        self.title.as_str()
    }

    pub fn content(&self) -> &str {
        self.content.as_str()
    }
}

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
    pub display_name: Option<String>,
    pub description: Option<String>,
}

pub type MediaTypes = Vec<String>;

pub struct RamlArgs {
    pub title: String,
    pub version: Option<String>,
    pub description: Option<String>,
    pub base_uri: Option<String>,
    pub protocols: Option<Vec<Protocol>>,
    pub media_types: Option<Vec<String>>,
    pub documentation: Option<Vec<RamlDocumentation>>,
    pub security_schemes: Option<SecuritySchemes>,
}

impl Raml {
    pub fn new(args: RamlArgs) -> Raml {
        Raml {
            title: args.title,
            version: args.version,
            description: args.description,
            base_uri: args.base_uri,
            protocols: args.protocols,
            media_types: args.media_types,
            documentation: args.documentation,
            security_schemes: args.security_schemes,
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

    pub fn protocols(self) -> Option<Protocols> {
        self.protocols
    }

    pub fn media_types(self) -> Option<MediaTypes> {
        self.media_types
    }

    pub fn documentation(self) -> Option<Vec<RamlDocumentation>> {
        self.documentation
    }

    pub fn security_schemes(self) -> Option<SecuritySchemes> {
        self.security_schemes
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

fn get_media_types(cursor: &mut ForwardCursor) -> Result<MediaTypes, RamlError> {
    let media_types = get_single_or_multiple_values(cursor)
        ?
        .iter()
        .map(|e| e.value.clone())
        .collect();
    Ok(media_types)
}

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
                Ok(RamlDocumentation::new(title.unwrap(), content.unwrap()))
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
    let mut display_name: Option<String> = None;
    let mut description: Option<String> = None;
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
                    TokenType::Scalar(_, ref v) if v == "displayName" => {
                        display_name = Some(get_single_value(cursor)?);
                    }
                    TokenType::Scalar(_, ref v) if v == "description" => {
                        description = Some(get_single_value(cursor)?);
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

    Ok(SecurityScheme {
        security_type: security_type.unwrap(),
        display_name: display_name,
        description: description,
    })
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
    Ok(Raml::new(RamlArgs {
        title: title.unwrap(),
        version: version,
        description: description,
        base_uri: base_uri,
        protocols: protocols,
        media_types: media_types,
        documentation: documentation,
        security_schemes: security_schemes,
    }))
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
