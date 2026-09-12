//! Simple template engine for notification messages.
//!
//! Uses Handlebars-style variable substitution with {{variable}} syntax.
//! No complex control flow — just variable replacement and conditionals.

use serde_json::Value;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TemplateError {
    #[error("Template rendering error: {0}")]
    Render(String),
    #[error("Missing variable: {0}")]
    MissingVariable(String),
}

/// Simple template engine supporting {{variable}} substitution.
#[derive(Debug, Clone, Default)]
pub struct TemplateEngine;

impl TemplateEngine {
    pub fn new() -> Self {
        Self
    }

    /// Render a template by substituting {{variable}} placeholders.
    ///
    /// # Arguments
    /// * `template` - Template string with {{variable}} placeholders
    /// * `data` - Key-value map of variable values
    ///
    /// # Example
    /// ```ignore
    /// let mut data = HashMap::new();
    /// data.insert("name", "John");
    /// data.insert("age", "30");
    /// let rendered = engine.render("Hello {{name}}, you are {{age}}", &data);
    /// ```
    pub fn render(
        &self,
        template: &str,
        data: &HashMap<String, String>,
    ) -> Result<String, TemplateError> {
        let mut result = template.to_string();
        for (key, value) in data {
            let placeholder = format!("{{{{{}}}}}", key);
            result = result.replace(&placeholder, value);
        }
        // Check for unreplaced placeholders
        if result.contains("{{") {
            // Extract the first missing variable name for error reporting
            if let Some(start) = result.find("{{") {
                if let Some(end) = result[start..].find("}}") {
                    let var = &result[start + 2..start + end].trim();
                    return Err(TemplateError::MissingVariable(var.to_string()));
                }
            }
        }
        Ok(result)
    }

    /// Render a template using a serde_json::Value as the data source.
    pub fn render_json(&self, template: &str, data: &Value) -> Result<String, TemplateError> {
        let mut result = template.to_string();

        if let Value::Object(map) = data {
            for (key, value) in map {
                let placeholder = format!("{{{{{}}}}}", key);
                let value_str = match value {
                    Value::String(s) => s.clone(),
                    Value::Number(n) => n.to_string(),
                    Value::Bool(b) => b.to_string(),
                    Value::Null => "null".to_string(),
                    _ => value.to_string(),
                };
                result = result.replace(&placeholder, &value_str);
            }
        }

        if result.contains("{{") {
            if let Some(start) = result.find("{{") {
                if let Some(end) = result[start..].find("}}") {
                    let var = &result[start + 2..start + end].trim();
                    return Err(TemplateError::MissingVariable(var.to_string()));
                }
            }
        }
        Ok(result)
    }
}
