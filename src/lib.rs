extern crate yaml_rust;

use yaml_rust::{YamlLoader, Yaml};

pub type RamlResult = Result<Raml, RamlErrors>;

pub fn parse(s: &str) -> RamlResult {
    let yaml = &(load_yaml(s)?);

    error_if_incorrect_raml_comment(s)?;

    parse_document_root(yaml)
}

fn parse_document_root(yaml: &Yaml) -> RamlResult {
    let mut raml_errors = RamlErrors::new();
    let mut title: Option<&str> = None;
    let mut version: Option<&str> = None;

    match *yaml {
        Yaml::Hash(ref h) => {
            for (k, v) in h {
                match *k {
                    Yaml::String(ref s) if s == "title" => {
                        title = v.as_str();
                    }
                    Yaml::String(ref s) if s == "version" => {
                        version = v.as_str();
                    }
                    Yaml::String(ref s) => {
                        raml_errors.add_error(format!("Unexpected field found at the document root: {}", s).as_str())
                    }
                    _ => {
                        // todo better error message
                        raml_errors.add_error("Invalid RAML")
                    }
                }
            }
        }
        _ => raml_errors.add_error("Unexpected YAML format"),
    }

    if title.is_none() {
        raml_errors.add_error("Error parsing document root. Missing field: title")
    }

    if raml_errors.has_errors() {
        Err(raml_errors)
    } else {
        Ok(Raml {
            title: title.unwrap().to_string(),
            version: version.map(|s| s.to_string()),
        })
    }
}

fn load_yaml(s: &str) -> Result<Yaml, RamlErrors> {
    let mut raml_errors = RamlErrors::new();
    let result = YamlLoader::load_from_str(s);
    match result {
        Ok(mut docs) => {
            if docs.is_empty() {
                raml_errors.add_error("Attempted to parse an empty document");
                Err(raml_errors)
            } else {
                Ok(docs.pop().unwrap())
            }
        }
        Err(scan_error) => {
            raml_errors.add_error(format!("Invalid yaml: {}", scan_error).as_str());
            Err(raml_errors)
        } 
    }
}

fn error_if_incorrect_raml_comment(s: &str) -> Result<(), RamlErrors> {
    let first_line: &str = s.lines().next().unwrap_or_default().trim();
    if first_line != "#%RAML 1.0" {
        let mut raml_errors = RamlErrors::new();
        raml_errors.errors
            .push(String::from("Document must start with the following RAML comment line: \
                                #%RAML 1.0"));
        return Err(raml_errors);
    }
    Ok(())
}

#[derive(Debug)]
#[derive(PartialEq)]
pub struct Raml {
    title: String,
    version: Option<String>,
}

impl Raml {
    pub fn title(&self) -> &str {
        self.title.as_str()
    }

    pub fn version(self) -> Option<String> {
        self.version
    }
}

#[derive(Default)]
#[derive(Debug)]
pub struct RamlErrors {
    errors: Vec<String>,
}

impl RamlErrors {
    pub fn new() -> RamlErrors {
        let errs = Vec::new();
        RamlErrors { errors: errs }
    }

    pub fn errors(&self) -> &Vec<String> {
        &self.errors
    }

    fn add_error(&mut self, error_message: &str) {
        self.errors.push(error_message.to_string())
    }

    fn has_errors(&self) -> bool {
        !&self.errors.is_empty()
    }
}
