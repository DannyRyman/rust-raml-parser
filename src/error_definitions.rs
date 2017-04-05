use token_type_definitions::TokenTypeDef;
use yaml_rust::scanner::{Marker, ScanError};

#[derive(Debug)]
pub enum ErrorDef {
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
    MissingProtocols,
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
        ErrorDef::MissingProtocols => {
            "Error parsing document root. Protocols must not be empty".to_string()
        }
    };
    match marker {
        Some(m) => RamlError::with_marker(message.as_str(), m),
        None => RamlError::new(message.as_str()),
    }
}