use std::collections::HashMap;

/// Simple template engine for prompt variable substitution
/// Supports {{variable}} syntax (Handlebars-style)
pub struct TemplateEngine;

impl TemplateEngine {
    /// Render a template by replacing {{variable}} placeholders with values
    ///
    /// # Arguments
    /// * `template` - The template string with {{variable}} placeholders
    /// * `variables` - A map of variable names to their values
    ///
    /// # Returns
    /// The rendered template with all placeholders replaced
    pub fn render(template: &str, variables: &HashMap<&str, &str>) -> String {
        let mut result = template.to_string();
        for (key, value) in variables {
            let placeholder = format!("{{{{{}}}}}", key);
            result = result.replace(&placeholder, value);
        }
        result
    }

    /// Extract all variable names used in a template
    /// Useful for validation and UI hints
    #[allow(dead_code)]
    pub fn extract_variables(template: &str) -> Vec<String> {
        let mut variables = Vec::new();
        let mut chars = template.chars().peekable();

        while let Some(c) = chars.next() {
            if c == '{' {
                if chars.peek() == Some(&'{') {
                    chars.next(); // consume second '{'
                    let mut var_name = String::new();
                    while let Some(&next_c) = chars.peek() {
                        if next_c == '}' {
                            chars.next();
                            if chars.peek() == Some(&'}') {
                                chars.next();
                                if !var_name.is_empty() && !variables.contains(&var_name) {
                                    variables.push(var_name);
                                }
                                break;
                            }
                        } else {
                            var_name.push(chars.next().unwrap());
                        }
                    }
                }
            }
        }

        variables
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_simple() {
        let mut vars = HashMap::new();
        vars.insert("name", "World");

        let result = TemplateEngine::render("Hello, {{name}}!", &vars);
        assert_eq!(result, "Hello, World!");
    }

    #[test]
    fn test_render_multiple_variables() {
        let mut vars = HashMap::new();
        vars.insert("transcript", "User: Hello\nAssistant: Hi there!");
        vars.insert("cwd", "/home/user/project");
        vars.insert("language", "en");

        let template = "Transcript:\n{{transcript}}\n\nDirectory: {{cwd}}\nLanguage: {{language}}";
        let result = TemplateEngine::render(template, &vars);

        assert!(result.contains("User: Hello"));
        assert!(result.contains("/home/user/project"));
        assert!(result.contains("Language: en"));
    }

    #[test]
    fn test_render_missing_variable() {
        let vars = HashMap::new();
        let result = TemplateEngine::render("Hello, {{name}}!", &vars);
        assert_eq!(result, "Hello, {{name}}!");
    }

    #[test]
    fn test_render_repeated_variable() {
        let mut vars = HashMap::new();
        vars.insert("x", "A");

        let result = TemplateEngine::render("{{x}} and {{x}}", &vars);
        assert_eq!(result, "A and A");
    }

    #[test]
    fn test_extract_variables() {
        let template = "Hello {{name}}, your {{item}} is ready. {{name}} again.";
        let vars = TemplateEngine::extract_variables(template);

        assert_eq!(vars.len(), 2);
        assert!(vars.contains(&"name".to_string()));
        assert!(vars.contains(&"item".to_string()));
    }

    #[test]
    fn test_extract_variables_empty() {
        let template = "No variables here";
        let vars = TemplateEngine::extract_variables(template);
        assert!(vars.is_empty());
    }
}
