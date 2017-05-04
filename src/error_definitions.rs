use yaml::TokenTypeDef;
use yaml_rust::scanner::{Marker, ScanError};
use std::fmt;

#[derive(Debug)]
pub enum HierarchyLevel {
    DocumentRoot,
    Documentation,
    SecurityScheme,
}

impl fmt::Display for HierarchyLevel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let printable = match *self {
            HierarchyLevel::DocumentRoot => "document root",
            HierarchyLevel::Documentation => "documentation",
            HierarchyLevel::SecurityScheme => "security scheme",
        };
        write!(f, "{}", printable)
    }
}

#[derive(Debug)]
pub enum ErrorDef {
    UnexpectedKeyRoot {
        field: String,
        level: HierarchyLevel,
    },
    UnexpectedEntry {
        expected: TokenTypeDef,
        found: TokenTypeDef,
    },
    UnexpectedEntryMulti {
        expected: Vec<TokenTypeDef>,
        found: TokenTypeDef,
    },
    MissingRamlVersion,
    MissingField {
        field: String,
        level: HierarchyLevel,
    },
    UnexpectedProtocol,
    MissingProtocols,
    InvalidSecuritySchemeType,
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

pub fn get_error(error: ErrorDef, marker: Option<Marker>) -> RamlError {
    let message = match error {
        ErrorDef::UnexpectedKeyRoot { field, level } => {
            format!("Unexpected field found at the {}: {}", level, field)
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
        ErrorDef::MissingField { field, level } => {
            format!("Error parsing {}. Missing field: {}", level, field)
        }
        ErrorDef::UnexpectedProtocol => {
            "Error parsing document root. Unexpected protocol".to_string()
        }
        ErrorDef::MissingProtocols => {
            "Error parsing document root. Protocols must not be empty".to_string()
        }
        ErrorDef::InvalidSecuritySchemeType => {
            "Error parsing security scheme. Unexpected type".to_string()
        }
    };
    match marker {
        Some(m) => RamlError::with_marker(message.as_str(), m),
        None => RamlError::new(message.as_str()),
    }
}