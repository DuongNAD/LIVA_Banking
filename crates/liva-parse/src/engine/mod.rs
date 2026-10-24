//! Engine module for template evaluation, sniffing, and parsing bank statements.
//! Strictly enforces `#![deny(clippy::float_arithmetic)]`.

#![deny(clippy::float_arithmetic)]

pub mod csv_engine;
pub mod date_normalizer;
pub mod excel_engine;
pub mod invariant;
pub mod number_normalizer;
pub mod sniffing;
pub mod swift_engine;
pub mod xml_engine;

use crate::builtin_templates::get_builtin_template_yamls;
use crate::error::ParseError;
use crate::template::schema::{ContainerFormat, StatementTemplate};
use crate::template::validator::validate_template;
use crate::{BankParser, CanonicalStatement};
use sniffing::{detect_container_format, evaluate_template_score};
use std::path::Path;

/// Registry of loaded statement templates supporting dynamic sniffing and retrieval.
#[derive(Debug, Clone, Default)]
pub struct TemplateRegistry {
    templates: Vec<StatementTemplate>,
}

impl TemplateRegistry {
    /// Creates an empty template registry.
    pub fn new() -> Self {
        Self {
            templates: Vec::new(),
        }
    }

    /// Creates a registry pre-loaded with all built-in bank templates.
    pub fn with_builtins() -> Result<Self, ParseError> {
        let mut registry = Self::new();
        for (name, yaml_content) in get_builtin_template_yamls() {
            let template: StatementTemplate = serde_yaml::from_str(yaml_content)
                .map_err(|e| ParseError::TemplateError(format!("Failed to parse built-in template '{name}': {e}")))?;
            validate_template(&template)?;
            registry.templates.push(template);
        }
        Ok(registry)
    }

    /// Registers and validates a new statement template.
    pub fn register(&mut self, template: StatementTemplate) -> Result<(), ParseError> {
        validate_template(&template)?;
        // Replace existing if bank_code and name match
        if let Some(pos) = self.templates.iter().position(|t| t.bank_code == template.bank_code && t.name == template.name) {
            self.templates[pos] = template;
        } else {
            self.templates.push(template);
        }
        Ok(())
    }

    /// Loads and registers a template from a YAML or JSON string.
    pub fn load_from_str(&mut self, content: &str) -> Result<(), ParseError> {
        let template: StatementTemplate = if content.trim_start().starts_with('{') {
            serde_json::from_str(content)
                .map_err(|e| ParseError::TemplateError(format!("JSON parse error: {e}")))?
        } else {
            serde_yaml::from_str(content)
                .map_err(|e| ParseError::TemplateError(format!("YAML parse error: {e}")))?
        };
        self.register(template)
    }

    /// Scans a directory and loads all .yaml, .yml, and .json templates.
    pub fn load_from_dir<P: AsRef<Path>>(&mut self, dir: P) -> Result<usize, ParseError> {
        let path = dir.as_ref();
        if !path.is_dir() {
            return Err(ParseError::IoError(format!("Directory does not exist: {}", path.display())));
        }

        let mut loaded = 0;
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let entry_path = entry.path();
            if let Some(ext) = entry_path.extension().and_then(|s| s.to_str()) {
                if ext.eq_ignore_ascii_case("yaml") || ext.eq_ignore_ascii_case("yml") || ext.eq_ignore_ascii_case("json") {
                    let content = std::fs::read_to_string(&entry_path)?;
                    self.load_from_str(&content)?;
                    loaded += 1;
                }
            }
        }

        Ok(loaded)
    }

    /// Evaluates all registered templates and returns the best matching template and its score.
    pub fn match_template<'a>(&'a self, raw: &[u8], filename: &str) -> Option<(&'a StatementTemplate, u32)> {
        let mut candidates = Vec::new();
        for t in &self.templates {
            let score = evaluate_template_score(t, raw, filename);
            if score >= t.sniffing.content_rules.min_score {
                candidates.push((t, score));
            }
        }
        candidates.into_iter().max_by_key(|(_, s)| *s)
    }

    /// Finds a registered template by bank code.
    pub fn find_by_bank_code(&self, bank_code: &str) -> Option<&StatementTemplate> {
        self.templates.iter().find(|t| t.bank_code.eq_ignore_ascii_case(bank_code))
    }

    /// Returns all registered templates.
    pub fn templates(&self) -> &[StatementTemplate] {
        &self.templates
    }

    /// Returns the number of registered templates.
    pub fn len(&self) -> usize {
        self.templates.len()
    }

    /// Returns true if the registry has no templates.
    pub fn is_empty(&self) -> bool {
        self.templates.is_empty()
    }
}

/// Dynamic statement parsing engine powered by StatementTemplate DSL.
#[derive(Debug, Clone)]
pub struct StatementEngine {
    registry: TemplateRegistry,
}

impl StatementEngine {
    /// Creates a new StatementEngine pre-loaded with built-in templates.
    pub fn new() -> Result<Self, ParseError> {
        let registry = TemplateRegistry::with_builtins()?;
        Ok(Self { registry })
    }

    /// Creates a StatementEngine with an empty template registry.
    pub fn empty() -> Self {
        Self {
            registry: TemplateRegistry::new(),
        }
    }

    /// Creates a StatementEngine wrapping a custom TemplateRegistry.
    pub fn from_registry(registry: TemplateRegistry) -> Self {
        Self { registry }
    }

    /// Access the underlying TemplateRegistry.
    pub fn registry(&self) -> &TemplateRegistry {
        &self.registry
    }

    /// Access the mutable TemplateRegistry.
    pub fn registry_mut(&mut self) -> &mut TemplateRegistry {
        &mut self.registry
    }

    /// Loads additional templates from a directory.
    pub fn load_templates<P: AsRef<Path>>(&mut self, dir: P) -> Result<usize, ParseError> {
        self.registry.load_from_dir(dir)
    }

    /// Parses raw statement payload using a specific template.
    pub fn parse_with_template(
        &self,
        raw: &[u8],
        filename: &str,
        template: &StatementTemplate,
    ) -> Result<CanonicalStatement, ParseError> {
        let format = detect_container_format(raw, filename);
        match format {
            ContainerFormat::ExcelZip | ContainerFormat::ExcelOle => {
                excel_engine::parse_excel_statement(raw, filename, template)
            }
            ContainerFormat::Xml => xml_engine::parse_xml_statement(raw, filename, template),
            ContainerFormat::SwiftMt => swift_engine::parse_swift_statement(raw, filename, template),
            _ => csv_engine::parse_csv_statement(raw, filename, template),
        }
    }

    /// Automatically sniffs and parses the raw statement payload using the top-scoring registered template.
    pub fn parse(&self, raw: &[u8], filename: &str) -> Result<CanonicalStatement, ParseError> {
        let (template, _score) = self
            .registry
            .match_template(raw, filename)
            .ok_or_else(|| {
                ParseError::UnsupportedFormat(format!(
                    "No matching bank template found for statement: '{filename}'"
                ))
            })?;

        self.parse_with_template(raw, filename, template)
    }
}

impl BankParser for StatementEngine {
    fn sniff(&self, raw: &[u8], filename: &str) -> bool {
        self.registry.match_template(raw, filename).is_some()
    }

    fn parse(&self, raw: &[u8], filename: &str) -> Result<CanonicalStatement, ParseError> {
        self.parse(raw, filename)
    }
}
