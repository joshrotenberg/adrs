//! Template system for generating ADR files.

use crate::{Adr, Config, Error, Result};
use minijinja::{Environment, context};
use std::path::Path;

/// Built-in template formats.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum TemplateFormat {
    /// Michael Nygard's original ADR format.
    #[default]
    Nygard,

    /// Markdown Any Decision Records format.
    Madr,
}

impl std::fmt::Display for TemplateFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Nygard => write!(f, "nygard"),
            Self::Madr => write!(f, "madr"),
        }
    }
}

impl std::str::FromStr for TemplateFormat {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "nygard" | "default" => Ok(Self::Nygard),
            "madr" => Ok(Self::Madr),
            _ => Err(Error::TemplateNotFound(s.to_string())),
        }
    }
}

/// A template for generating ADRs.
#[derive(Debug, Clone)]
pub struct Template {
    /// The template content.
    content: String,

    /// The template name (for error messages).
    name: String,
}

impl Template {
    /// Create a template from a string.
    pub fn from_string(name: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            content: content.into(),
        }
    }

    /// Load a template from a file.
    pub fn from_file(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("custom")
            .to_string();
        Ok(Self { name, content })
    }

    /// Get a built-in template by format.
    pub fn builtin(format: TemplateFormat) -> Self {
        match format {
            TemplateFormat::Nygard => Self::from_string("nygard", NYGARD_TEMPLATE),
            TemplateFormat::Madr => Self::from_string("madr", MADR_TEMPLATE),
        }
    }

    /// Render the template with the given ADR data.
    pub fn render(&self, adr: &Adr, config: &Config) -> Result<String> {
        use crate::LinkKind;

        let mut env = Environment::new();
        env.add_template(&self.name, &self.content)
            .map_err(|e| Error::TemplateError(e.to_string()))?;

        let template = env
            .get_template(&self.name)
            .map_err(|e| Error::TemplateError(e.to_string()))?;

        // Convert links to a format with display-friendly kind
        let links: Vec<_> = adr
            .links
            .iter()
            .map(|link| {
                let kind_display = match &link.kind {
                    LinkKind::Supersedes => "Supersedes",
                    LinkKind::SupersededBy => "Superseded by",
                    LinkKind::Amends => "Amends",
                    LinkKind::AmendedBy => "Amended by",
                    LinkKind::RelatesTo => "Relates to",
                    LinkKind::Custom(s) => s.as_str(),
                };
                context! {
                    target => link.target,
                    kind => kind_display,
                    description => &link.description,
                }
            })
            .collect();

        let output = template
            .render(context! {
                number => adr.number,
                title => &adr.title,
                date => crate::parse::format_date(adr.date),
                status => adr.status.to_string(),
                context => &adr.context,
                decision => &adr.decision,
                consequences => &adr.consequences,
                links => links,
                is_ng => config.is_next_gen(),
            })
            .map_err(|e| Error::TemplateError(e.to_string()))?;

        Ok(output)
    }
}

/// Template engine for managing and rendering templates.
#[derive(Debug)]
pub struct TemplateEngine {
    /// The default template format.
    default_format: TemplateFormat,

    /// Custom template path (overrides built-in).
    custom_template: Option<Template>,
}

impl Default for TemplateEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl TemplateEngine {
    /// Create a new template engine.
    pub fn new() -> Self {
        Self {
            default_format: TemplateFormat::default(),
            custom_template: None,
        }
    }

    /// Set the default template format.
    pub fn with_format(mut self, format: TemplateFormat) -> Self {
        self.default_format = format;
        self
    }

    /// Set a custom template.
    pub fn with_custom_template(mut self, template: Template) -> Self {
        self.custom_template = Some(template);
        self
    }

    /// Load a custom template from a file.
    pub fn with_custom_template_file(mut self, path: &Path) -> Result<Self> {
        self.custom_template = Some(Template::from_file(path)?);
        Ok(self)
    }

    /// Get the template to use for rendering.
    pub fn template(&self) -> Template {
        self.custom_template
            .clone()
            .unwrap_or_else(|| Template::builtin(self.default_format))
    }

    /// Render an ADR using the configured template.
    pub fn render(&self, adr: &Adr, config: &Config) -> Result<String> {
        self.template().render(adr, config)
    }
}

/// Nygard's original ADR template (compatible mode).
const NYGARD_TEMPLATE: &str = r#"{% if is_ng %}---
number: {{ number }}
title: {{ title }}
date: {{ date }}
status: {{ status | lower }}
{% if links %}links:
{% for link in links %}  - target: {{ link.target }}
    kind: {{ link.kind | lower }}
{% endfor %}{% endif %}---

{% endif %}# {{ number }}. {{ title }}

Date: {{ date }}

## Status

{{ status }}
{% for link in links %}
{{ link.kind }} [{{ link.target }}. ...]({{ "%04d" | format(link.target) }}-....md)
{% endfor %}
## Context

{{ context if context else "What is the issue that we're seeing that is motivating this decision or change?" }}

## Decision

{{ decision if decision else "What is the change that we're proposing and/or doing?" }}

## Consequences

{{ consequences if consequences else "What becomes easier or more difficult to do because of this change?" }}
"#;

/// MADR (Markdown Any Decision Records) template.
const MADR_TEMPLATE: &str = r#"{% if is_ng %}---
number: {{ number }}
title: {{ title }}
date: {{ date }}
status: {{ status | lower }}
{% if links %}links:
{% for link in links %}  - target: {{ link.target }}
    kind: {{ link.kind | lower }}
{% endfor %}{% endif %}---

{% endif %}# {{ title }}

* Status: {{ status }}
* Date: {{ date }}

## Context and Problem Statement

{{ context if context else "Describe the context and problem statement, e.g., in free form using two to three sentences or in the form of an illustrative story. You may want to articulate the problem in form of a question and add links to collaboration boards or issue management systems." }}

## Decision Drivers

* {decision driver 1, e.g., a force, facing concern, ...}
* {decision driver 2, e.g., a force, facing concern, ...}
* ...

## Considered Options

* {title of option 1}
* {title of option 2}
* {title of option 3}
* ...

## Decision Outcome

{{ decision if decision else "Chosen option: \"{title of option 1}\", because {justification. e.g., only option, which meets k.o. criterion decision driver | which resolves force {force} | ... | comes out best (see below)}." }}

### Consequences

{{ consequences if consequences else "* Good, because {positive consequence, e.g., improvement of one or more desired qualities, ...}\n* Bad, because {negative consequence, e.g., compromising one or more desired qualities, ...}" }}

## More Information

{You might want to provide additional evidence/confidence for the decision outcome here and/or document the team agreement on the decision and/or define when/how this decision should be realized and if/when it should be re-visited. Links to other decisions and resources might appear here as well.}
"#;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AdrStatus, ConfigMode};

    #[test]
    fn test_render_nygard_compatible() {
        let template = Template::builtin(TemplateFormat::Nygard);
        let mut adr = Adr::new(1, "Use Rust");
        adr.status = AdrStatus::Accepted;

        let config = Config::default();
        let output = template.render(&adr, &config).unwrap();

        assert!(output.contains("# 1. Use Rust"));
        assert!(output.contains("## Status"));
        assert!(output.contains("Accepted"));
        assert!(!output.starts_with("---")); // No frontmatter in compatible mode
    }

    #[test]
    fn test_render_nygard_ng() {
        let template = Template::builtin(TemplateFormat::Nygard);
        let mut adr = Adr::new(1, "Use Rust");
        adr.status = AdrStatus::Accepted;

        let config = Config {
            mode: ConfigMode::NextGen,
            ..Default::default()
        };
        let output = template.render(&adr, &config).unwrap();

        assert!(output.starts_with("---")); // Has frontmatter in ng mode
        assert!(output.contains("number: 1"));
        assert!(output.contains("status: accepted"));
    }

    #[test]
    fn test_template_format_parse() {
        assert_eq!(
            "nygard".parse::<TemplateFormat>().unwrap(),
            TemplateFormat::Nygard
        );
        assert_eq!(
            "madr".parse::<TemplateFormat>().unwrap(),
            TemplateFormat::Madr
        );
        assert!("unknown".parse::<TemplateFormat>().is_err());
    }
}
