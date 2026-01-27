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

    /// Markdown Any Decision Records format (MADR 4.0.0).
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

/// Template variants for different levels of detail.
///
/// The variant names follow the MADR naming convention:
/// - **Full**: All sections with guidance text
/// - **Minimal**: Core sections only, with guidance text
/// - **Bare**: All sections, but empty (no guidance)
/// - **BareMinimal**: Core sections only, empty (no guidance)
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum TemplateVariant {
    /// Full template with all sections and guidance.
    #[default]
    Full,

    /// Minimal template with essential sections only (with guidance).
    Minimal,

    /// Bare template - all sections but empty/placeholder content.
    Bare,

    /// Bare-minimal template - fewest sections, empty content.
    BareMinimal,
}

impl std::fmt::Display for TemplateVariant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Full => write!(f, "full"),
            Self::Minimal => write!(f, "minimal"),
            Self::Bare => write!(f, "bare"),
            Self::BareMinimal => write!(f, "bare-minimal"),
        }
    }
}

impl std::str::FromStr for TemplateVariant {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().replace('_', "-").as_str() {
            "full" | "default" => Ok(Self::Full),
            "minimal" | "min" => Ok(Self::Minimal),
            "bare" => Ok(Self::Bare),
            "bare-minimal" | "bareminimal" | "empty" => Ok(Self::BareMinimal),
            _ => Err(Error::TemplateNotFound(format!("Unknown variant: {s}"))),
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

    /// Get the template content.
    pub fn content(&self) -> &str {
        &self.content
    }

    /// Get the template name.
    pub fn name(&self) -> &str {
        &self.name
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

    /// Get a built-in template by format (uses Full variant).
    pub fn builtin(format: TemplateFormat) -> Self {
        Self::builtin_with_variant(format, TemplateVariant::Full)
    }

    /// Get a built-in template by format and variant.
    pub fn builtin_with_variant(format: TemplateFormat, variant: TemplateVariant) -> Self {
        let (name, content) = match (format, variant) {
            // Nygard templates
            (TemplateFormat::Nygard, TemplateVariant::Full) => ("nygard", NYGARD_TEMPLATE),
            (TemplateFormat::Nygard, TemplateVariant::Minimal) => {
                ("nygard-minimal", NYGARD_MINIMAL_TEMPLATE)
            }
            (TemplateFormat::Nygard, TemplateVariant::Bare) => {
                ("nygard-bare", NYGARD_BARE_TEMPLATE)
            }
            (TemplateFormat::Nygard, TemplateVariant::BareMinimal) => {
                ("nygard-bare-minimal", NYGARD_BARE_MINIMAL_TEMPLATE)
            }

            // MADR templates
            (TemplateFormat::Madr, TemplateVariant::Full) => ("madr", MADR_TEMPLATE),
            (TemplateFormat::Madr, TemplateVariant::Minimal) => {
                ("madr-minimal", MADR_MINIMAL_TEMPLATE)
            }
            (TemplateFormat::Madr, TemplateVariant::Bare) => ("madr-bare", MADR_BARE_TEMPLATE),
            (TemplateFormat::Madr, TemplateVariant::BareMinimal) => {
                ("madr-bare-minimal", MADR_BARE_MINIMAL_TEMPLATE)
            }
        };
        Self::from_string(name, content)
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
                // MADR 4.0.0 fields
                decision_makers => &adr.decision_makers,
                consulted => &adr.consulted,
                informed => &adr.informed,
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

    /// The default template variant.
    default_variant: TemplateVariant,

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
            default_variant: TemplateVariant::default(),
            custom_template: None,
        }
    }

    /// Set the default template format.
    pub fn with_format(mut self, format: TemplateFormat) -> Self {
        self.default_format = format;
        self
    }

    /// Set the default template variant.
    pub fn with_variant(mut self, variant: TemplateVariant) -> Self {
        self.default_variant = variant;
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
        self.custom_template.clone().unwrap_or_else(|| {
            Template::builtin_with_variant(self.default_format, self.default_variant)
        })
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

/// MADR (Markdown Any Decision Records) 4.0.0 template.
const MADR_TEMPLATE: &str = r#"---
number: {{ number }}
title: {{ title }}
status: {{ status | lower }}
date: {{ date }}
{% if decision_makers %}decision-makers:
{% for dm in decision_makers %}  - {{ dm }}
{% endfor %}{% endif %}{% if consulted %}consulted:
{% for c in consulted %}  - {{ c }}
{% endfor %}{% endif %}{% if informed %}informed:
{% for i in informed %}  - {{ i }}
{% endfor %}{% endif %}---

# {{ title }}

## Context and Problem Statement

{{ context if context else "{Describe the context and problem statement, e.g., in free form using two to three sentences or in the form of an illustrative story. You may want to articulate the problem in form of a question and add links to collaboration boards or issue management systems.}" }}

<!-- This is an optional element. Feel free to remove. -->
## Decision Drivers

* {decision driver 1, e.g., a force, facing concern, ...}
* {decision driver 2, e.g., a force, facing concern, ...}
* ... <!-- numbers of drivers can vary -->

## Considered Options

* {title of option 1}
* {title of option 2}
* {title of option 3}
* ... <!-- numbers of options can vary -->

## Decision Outcome

{{ decision if decision else "Chosen option: \"{title of option 1}\", because {justification. e.g., only option, which meets k.o. criterion decision driver | which resolves force {force} | ... | comes out best (see below)}." }}

<!-- This is an optional element. Feel free to remove. -->
### Consequences

{{ consequences if consequences else "* Good, because {positive consequence, e.g., improvement of one or more desired qualities, ...}\n* Bad, because {negative consequence, e.g., compromising one or more desired qualities, ...}\n* ... <!-- numbers of consequences can vary -->" }}

<!-- This is an optional element. Feel free to remove. -->
### Confirmation

{Describe how the implementation/compliance of the ADR can/will be confirmed. Is there any automated or manual fitness function? If so, list it and explain how it is applied. Is the chosen design and its implementation in line with the decision? E.g., a design/code review or a test with a library such as ArchUnit can help validate this. Note that although we classify this element as optional, it is included in many ADRs.}

<!-- This is an optional element. Feel free to remove. -->
## Pros and Cons of the Options

### {title of option 1}

<!-- This is an optional element. Feel free to remove. -->
{example | description | pointer to more information | ...}

* Good, because {argument a}
* Good, because {argument b}
<!-- use "neutral" if the given argument weights neither for good nor bad -->
* Neutral, because {argument c}
* Bad, because {argument d}
* ... <!-- numbers of pros and cons can vary -->

### {title of other option}

{example | description | pointer to more information | ...}

* Good, because {argument a}
* Good, because {argument b}
* Neutral, because {argument c}
* Bad, because {argument d}
* ...

<!-- This is an optional element. Feel free to remove. -->
## More Information

{You might want to provide additional evidence/confidence for the decision outcome here and/or document the team agreement on the decision and/or define when/how this decision should be realized and if/when it should be re-visited. Links to other decisions and resources might appear here as well.}
"#;

/// Nygard minimal template - essential sections only.
const NYGARD_MINIMAL_TEMPLATE: &str = r#"{% if is_ng %}---
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

{{ context if context else "" }}

## Decision

{{ decision if decision else "" }}

## Consequences

{{ consequences if consequences else "" }}
"#;

/// Nygard bare template - just the structure, no guidance.
const NYGARD_BARE_TEMPLATE: &str = r#"# {{ number }}. {{ title }}

Date: {{ date }}

## Status

{{ status }}

## Context



## Decision



## Consequences

"#;

/// Nygard bare-minimal template - fewest sections, empty content.
const NYGARD_BARE_MINIMAL_TEMPLATE: &str = r#"# {{ number }}. {{ title }}

## Status

{{ status }}

## Context



## Decision



## Consequences

"#;

/// MADR minimal template - core sections only, no frontmatter.
/// Matches official MADR adr-template-minimal.md
const MADR_MINIMAL_TEMPLATE: &str = r#"# {{ title }}

## Context and Problem Statement

{{ context if context else "{Describe the context and problem statement, e.g., in free form using two to three sentences or in the form of an illustrative story. You may want to articulate the problem in form of a question and add links to collaboration boards or issue management systems.}" }}

## Considered Options

* {title of option 1}
* {title of option 2}
* {title of option 3}
* ... <!-- numbers of options can vary -->

## Decision Outcome

{{ decision if decision else "Chosen option: \"{title of option 1}\", because {justification. e.g., only option, which meets k.o. criterion decision driver | which resolves force {force} | ... | comes out best (see below)}." }}

<!-- This is an optional element. Feel free to remove. -->
### Consequences

{{ consequences if consequences else "* Good, because {positive consequence, e.g., improvement of one or more desired qualities, ...}\n* Bad, because {negative consequence, e.g., compromising one or more desired qualities, ...}\n* ... <!-- numbers of consequences can vary -->" }}
"#;

/// MADR bare template - all sections with empty placeholders.
/// Matches official MADR adr-template-bare.md
const MADR_BARE_TEMPLATE: &str = r#"---
number: {{ number }}
title: {{ title }}
status: {{ status | lower }}
date: {{ date }}
decision-makers:
consulted:
informed:
---

# {{ title }}

## Context and Problem Statement



## Decision Drivers

* <!-- decision driver -->

## Considered Options

* <!-- option -->

## Decision Outcome

Chosen option: "", because

### Consequences

* Good, because
* Bad, because

### Confirmation



## Pros and Cons of the Options

### <!-- title of option -->

* Good, because
* Neutral, because
* Bad, because

## More Information

"#;

/// MADR bare-minimal template - fewest sections, empty content.
/// Matches official MADR adr-template-bare-minimal.md
const MADR_BARE_MINIMAL_TEMPLATE: &str = r#"# {{ title }}

## Context and Problem Statement



## Considered Options



## Decision Outcome



### Consequences

"#;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AdrLink, AdrStatus, ConfigMode, LinkKind};
    use tempfile::TempDir;
    use test_case::test_case;

    // ========== TemplateFormat Tests ==========

    #[test]
    fn test_template_format_default() {
        assert_eq!(TemplateFormat::default(), TemplateFormat::Nygard);
    }

    #[test_case("nygard" => TemplateFormat::Nygard; "nygard")]
    #[test_case("Nygard" => TemplateFormat::Nygard; "nygard capitalized")]
    #[test_case("NYGARD" => TemplateFormat::Nygard; "nygard uppercase")]
    #[test_case("default" => TemplateFormat::Nygard; "default alias")]
    #[test_case("madr" => TemplateFormat::Madr; "madr")]
    #[test_case("MADR" => TemplateFormat::Madr; "madr uppercase")]
    fn test_template_format_parse(input: &str) -> TemplateFormat {
        input.parse().unwrap()
    }

    #[test]
    fn test_template_format_parse_unknown() {
        let result: Result<TemplateFormat> = "unknown".parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_template_format_display() {
        assert_eq!(TemplateFormat::Nygard.to_string(), "nygard");
        assert_eq!(TemplateFormat::Madr.to_string(), "madr");
    }

    // ========== TemplateVariant Tests ==========

    #[test]
    fn test_template_variant_default() {
        assert_eq!(TemplateVariant::default(), TemplateVariant::Full);
    }

    #[test_case("full" => TemplateVariant::Full; "full")]
    #[test_case("Full" => TemplateVariant::Full; "full capitalized")]
    #[test_case("default" => TemplateVariant::Full; "default alias")]
    #[test_case("minimal" => TemplateVariant::Minimal; "minimal")]
    #[test_case("min" => TemplateVariant::Minimal; "min alias")]
    #[test_case("bare" => TemplateVariant::Bare; "bare")]
    #[test_case("bare-minimal" => TemplateVariant::BareMinimal; "bare-minimal")]
    #[test_case("bareminimal" => TemplateVariant::BareMinimal; "bareminimal")]
    #[test_case("empty" => TemplateVariant::BareMinimal; "empty alias")]
    fn test_template_variant_parse(input: &str) -> TemplateVariant {
        input.parse().unwrap()
    }

    #[test]
    fn test_template_variant_parse_unknown() {
        let result: Result<TemplateVariant> = "unknown".parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_template_variant_display() {
        assert_eq!(TemplateVariant::Full.to_string(), "full");
        assert_eq!(TemplateVariant::Minimal.to_string(), "minimal");
        assert_eq!(TemplateVariant::Bare.to_string(), "bare");
        assert_eq!(TemplateVariant::BareMinimal.to_string(), "bare-minimal");
    }

    // ========== Template Creation Tests ==========

    #[test]
    fn test_template_from_string() {
        let template = Template::from_string("test", "# {{ title }}");
        assert_eq!(template.name, "test");
        assert_eq!(template.content, "# {{ title }}");
    }

    #[test]
    fn test_template_from_file() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("custom.md");
        std::fs::write(&path, "# {{ number }}. {{ title }}").unwrap();

        let template = Template::from_file(&path).unwrap();
        assert_eq!(template.name, "custom.md");
        assert!(template.content.contains("{{ number }}"));
    }

    #[test]
    fn test_template_from_file_not_found() {
        let result = Template::from_file(Path::new("/nonexistent/template.md"));
        assert!(result.is_err());
    }

    #[test]
    fn test_template_builtin_nygard() {
        let template = Template::builtin(TemplateFormat::Nygard);
        assert_eq!(template.name, "nygard");
        assert!(template.content.contains("## Status"));
        assert!(template.content.contains("## Context"));
        assert!(template.content.contains("## Decision"));
        assert!(template.content.contains("## Consequences"));
    }

    #[test]
    fn test_template_builtin_madr() {
        let template = Template::builtin(TemplateFormat::Madr);
        assert_eq!(template.name, "madr");
        assert!(template.content.contains("Context and Problem Statement"));
        assert!(template.content.contains("Decision Drivers"));
        assert!(template.content.contains("Considered Options"));
        assert!(template.content.contains("Decision Outcome"));
    }

    // ========== Template Rendering - Nygard Compatible Mode ==========

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
    fn test_render_nygard_all_statuses() {
        let template = Template::builtin(TemplateFormat::Nygard);
        let config = Config::default();

        for (status, expected_text) in [
            (AdrStatus::Proposed, "Proposed"),
            (AdrStatus::Accepted, "Accepted"),
            (AdrStatus::Deprecated, "Deprecated"),
            (AdrStatus::Superseded, "Superseded"),
            (AdrStatus::Custom("Draft".into()), "Draft"),
        ] {
            let mut adr = Adr::new(1, "Test");
            adr.status = status;

            let output = template.render(&adr, &config).unwrap();
            assert!(
                output.contains(expected_text),
                "Output should contain '{expected_text}': {output}"
            );
        }
    }

    #[test]
    fn test_render_nygard_with_content() {
        let template = Template::builtin(TemplateFormat::Nygard);
        let mut adr = Adr::new(1, "Use Rust");
        adr.status = AdrStatus::Accepted;
        adr.context = "We need a safe language.".to_string();
        adr.decision = "We will use Rust.".to_string();
        adr.consequences = "Better memory safety.".to_string();

        let config = Config::default();
        let output = template.render(&adr, &config).unwrap();

        assert!(output.contains("We need a safe language."));
        assert!(output.contains("We will use Rust."));
        assert!(output.contains("Better memory safety."));
    }

    #[test]
    fn test_render_nygard_with_links() {
        let template = Template::builtin(TemplateFormat::Nygard);
        let mut adr = Adr::new(2, "Use PostgreSQL");
        adr.status = AdrStatus::Accepted;
        adr.links.push(AdrLink::new(1, LinkKind::Supersedes));

        let config = Config::default();
        let output = template.render(&adr, &config).unwrap();

        assert!(output.contains("Supersedes"));
        assert!(output.contains("[1. ...]"));
        assert!(output.contains("0001-....md"));
    }

    #[test]
    fn test_render_nygard_with_multiple_links() {
        let template = Template::builtin(TemplateFormat::Nygard);
        let mut adr = Adr::new(5, "Combined Decision");
        adr.status = AdrStatus::Accepted;
        adr.links.push(AdrLink::new(1, LinkKind::Supersedes));
        adr.links.push(AdrLink::new(2, LinkKind::Amends));
        adr.links.push(AdrLink::new(3, LinkKind::SupersededBy));

        let config = Config::default();
        let output = template.render(&adr, &config).unwrap();

        assert!(output.contains("Supersedes"));
        assert!(output.contains("Amends"));
        assert!(output.contains("Superseded by"));
    }

    // ========== Template Rendering - Nygard NextGen Mode ==========

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
        assert!(output.contains("title: Use Rust"));
        assert!(output.contains("status: accepted"));
    }

    #[test]
    fn test_render_nygard_ng_with_links() {
        let template = Template::builtin(TemplateFormat::Nygard);
        let mut adr = Adr::new(2, "Test");
        adr.status = AdrStatus::Accepted;
        adr.links.push(AdrLink::new(1, LinkKind::Supersedes));

        let config = Config {
            mode: ConfigMode::NextGen,
            ..Default::default()
        };
        let output = template.render(&adr, &config).unwrap();

        assert!(output.contains("links:"));
        assert!(output.contains("target: 1"));
    }

    // ========== Template Rendering - MADR 4.0.0 ==========

    #[test]
    fn test_render_madr_basic() {
        let template = Template::builtin(TemplateFormat::Madr);
        let mut adr = Adr::new(1, "Use Rust");
        adr.status = AdrStatus::Accepted;

        let config = Config::default();
        let output = template.render(&adr, &config).unwrap();

        assert!(output.starts_with("---")); // MADR always has frontmatter
        assert!(output.contains("status: accepted"));
        assert!(output.contains("# Use Rust"));
        assert!(output.contains("## Context and Problem Statement"));
        assert!(output.contains("## Decision Drivers"));
        assert!(output.contains("## Considered Options"));
        assert!(output.contains("## Decision Outcome"));
        assert!(output.contains("## Pros and Cons of the Options"));
    }

    #[test]
    fn test_render_madr_with_decision_makers() {
        let template = Template::builtin(TemplateFormat::Madr);
        let mut adr = Adr::new(1, "Use Rust");
        adr.status = AdrStatus::Accepted;
        adr.decision_makers = vec!["Alice".into(), "Bob".into()];

        let config = Config::default();
        let output = template.render(&adr, &config).unwrap();

        assert!(output.contains("decision-makers:"));
        assert!(output.contains("  - Alice"));
        assert!(output.contains("  - Bob"));
    }

    #[test]
    fn test_render_madr_with_consulted() {
        let template = Template::builtin(TemplateFormat::Madr);
        let mut adr = Adr::new(1, "Use Rust");
        adr.status = AdrStatus::Accepted;
        adr.consulted = vec!["Carol".into()];

        let config = Config::default();
        let output = template.render(&adr, &config).unwrap();

        assert!(output.contains("consulted:"));
        assert!(output.contains("  - Carol"));
    }

    #[test]
    fn test_render_madr_with_informed() {
        let template = Template::builtin(TemplateFormat::Madr);
        let mut adr = Adr::new(1, "Use Rust");
        adr.status = AdrStatus::Accepted;
        adr.informed = vec!["Dave".into(), "Eve".into()];

        let config = Config::default();
        let output = template.render(&adr, &config).unwrap();

        assert!(output.contains("informed:"));
        assert!(output.contains("  - Dave"));
        assert!(output.contains("  - Eve"));
    }

    #[test]
    fn test_render_madr_full_frontmatter() {
        let template = Template::builtin(TemplateFormat::Madr);
        let mut adr = Adr::new(1, "Use MADR Format");
        adr.status = AdrStatus::Accepted;
        adr.decision_makers = vec!["Alice".into(), "Bob".into()];
        adr.consulted = vec!["Carol".into()];
        adr.informed = vec!["Dave".into()];

        let config = Config::default();
        let output = template.render(&adr, &config).unwrap();

        // Check frontmatter structure - now includes number and title
        assert!(
            output.starts_with("---\nnumber: 1\ntitle: Use MADR Format\nstatus: accepted\ndate:")
        );
        assert!(output.contains("decision-makers:\n  - Alice\n  - Bob"));
        assert!(output.contains("consulted:\n  - Carol"));
        assert!(output.contains("informed:\n  - Dave"));
        assert!(output.contains("---\n\n# Use MADR Format"));
    }

    #[test]
    fn test_render_madr_empty_optional_fields() {
        let template = Template::builtin(TemplateFormat::Madr);
        let mut adr = Adr::new(1, "Simple ADR");
        adr.status = AdrStatus::Proposed;

        let config = Config::default();
        let output = template.render(&adr, &config).unwrap();

        // Empty optional fields should not appear
        assert!(!output.contains("decision-makers:"));
        assert!(!output.contains("consulted:"));
        assert!(!output.contains("informed:"));
    }

    // ========== Template Variants Tests ==========

    #[test]
    fn test_nygard_minimal_template() {
        let template =
            Template::builtin_with_variant(TemplateFormat::Nygard, TemplateVariant::Minimal);
        let adr = Adr::new(1, "Minimal Test");
        let config = Config::default();
        let output = template.render(&adr, &config).unwrap();

        // Should have basic structure but no guidance text
        assert!(output.contains("# 1. Minimal Test"));
        assert!(output.contains("## Status"));
        assert!(output.contains("## Context"));
        assert!(output.contains("## Decision"));
        assert!(output.contains("## Consequences"));
        // Should NOT have guidance text
        assert!(!output.contains("What is the issue"));
    }

    #[test]
    fn test_nygard_bare_template() {
        let template =
            Template::builtin_with_variant(TemplateFormat::Nygard, TemplateVariant::Bare);
        let adr = Adr::new(1, "Bare Test");
        let config = Config::default();
        let output = template.render(&adr, &config).unwrap();

        // Should have basic structure
        assert!(output.contains("# 1. Bare Test"));
        assert!(output.contains("## Status"));
        assert!(output.contains("## Context"));
        // Bare template has no frontmatter
        assert!(!output.contains("---"));
    }

    #[test]
    fn test_madr_minimal_template() {
        let template =
            Template::builtin_with_variant(TemplateFormat::Madr, TemplateVariant::Minimal);
        let adr = Adr::new(1, "MADR Minimal");
        let config = Config::default();
        let output = template.render(&adr, &config).unwrap();

        // MADR minimal has NO frontmatter (matches official adr-template-minimal.md)
        assert!(!output.starts_with("---"));
        assert!(output.contains("# MADR Minimal"));
        assert!(output.contains("## Context and Problem Statement"));
        assert!(output.contains("## Considered Options"));
        assert!(output.contains("## Decision Outcome"));
        // Should NOT have full MADR sections
        assert!(!output.contains("## Decision Drivers"));
        assert!(!output.contains("## Pros and Cons"));
    }

    #[test]
    fn test_madr_bare_template() {
        let template = Template::builtin_with_variant(TemplateFormat::Madr, TemplateVariant::Bare);
        let adr = Adr::new(1, "MADR Bare");
        let config = Config::default();
        let output = template.render(&adr, &config).unwrap();

        // MADR bare has frontmatter with empty fields (matches official adr-template-bare.md)
        assert!(output.starts_with("---"));
        assert!(output.contains("status:"));
        assert!(output.contains("decision-makers:"));
        assert!(output.contains("consulted:"));
        assert!(output.contains("informed:"));
        assert!(output.contains("# MADR Bare"));
        // Should have ALL sections (empty)
        assert!(output.contains("## Decision Drivers"));
        assert!(output.contains("## Considered Options"));
        assert!(output.contains("## Pros and Cons of the Options"));
        assert!(output.contains("## More Information"));
    }

    #[test]
    fn test_madr_bare_minimal_template() {
        let template =
            Template::builtin_with_variant(TemplateFormat::Madr, TemplateVariant::BareMinimal);
        let adr = Adr::new(1, "MADR Bare Minimal");
        let config = Config::default();
        let output = template.render(&adr, &config).unwrap();

        // MADR bare-minimal has NO frontmatter, minimal sections
        assert!(!output.starts_with("---"));
        assert!(output.contains("# MADR Bare Minimal"));
        assert!(output.contains("## Context and Problem Statement"));
        assert!(output.contains("## Considered Options"));
        assert!(output.contains("## Decision Outcome"));
        assert!(output.contains("### Consequences"));
        // Should NOT have extended sections
        assert!(!output.contains("## Decision Drivers"));
        assert!(!output.contains("## Pros and Cons"));
    }

    #[test]
    fn test_nygard_bare_minimal_template() {
        let template =
            Template::builtin_with_variant(TemplateFormat::Nygard, TemplateVariant::BareMinimal);
        let adr = Adr::new(1, "Nygard Bare Minimal");
        let config = Config::default();
        let output = template.render(&adr, &config).unwrap();

        // Should have basic structure without Date line
        assert!(output.contains("# 1. Nygard Bare Minimal"));
        assert!(output.contains("## Status"));
        assert!(output.contains("## Context"));
        assert!(output.contains("## Decision"));
        assert!(output.contains("## Consequences"));
        // No frontmatter, no date
        assert!(!output.contains("---"));
        assert!(!output.contains("Date:"));
    }

    #[test]
    fn test_builtin_defaults_to_full() {
        let full = Template::builtin(TemplateFormat::Nygard);
        let explicit_full =
            Template::builtin_with_variant(TemplateFormat::Nygard, TemplateVariant::Full);

        assert_eq!(full.name, explicit_full.name);
        assert_eq!(full.content, explicit_full.content);
    }

    // ========== Template Engine Tests ==========

    #[test]
    fn test_template_engine_new() {
        let engine = TemplateEngine::new();
        assert_eq!(engine.default_format, TemplateFormat::Nygard);
        assert_eq!(engine.default_variant, TemplateVariant::Full);
        assert!(engine.custom_template.is_none());
    }

    #[test]
    fn test_template_engine_default() {
        let engine = TemplateEngine::default();
        assert_eq!(engine.default_format, TemplateFormat::Nygard);
        assert_eq!(engine.default_variant, TemplateVariant::Full);
    }

    #[test]
    fn test_template_engine_with_format() {
        let engine = TemplateEngine::new().with_format(TemplateFormat::Madr);
        assert_eq!(engine.default_format, TemplateFormat::Madr);
    }

    #[test]
    fn test_template_engine_with_custom_template() {
        let custom = Template::from_string("custom", "# {{ title }}");
        let engine = TemplateEngine::new().with_custom_template(custom);
        assert!(engine.custom_template.is_some());
    }

    #[test]
    fn test_template_engine_with_custom_template_file() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("template.md");
        std::fs::write(&path, "# {{ title }}").unwrap();

        let engine = TemplateEngine::new()
            .with_custom_template_file(&path)
            .unwrap();
        assert!(engine.custom_template.is_some());
    }

    #[test]
    fn test_template_engine_with_custom_template_file_not_found() {
        let result = TemplateEngine::new().with_custom_template_file(Path::new("/nonexistent.md"));
        assert!(result.is_err());
    }

    #[test]
    fn test_template_engine_template_builtin() {
        let engine = TemplateEngine::new();
        let template = engine.template();
        assert_eq!(template.name, "nygard");
    }

    #[test]
    fn test_template_engine_template_custom() {
        let custom = Template::from_string("my-template", "# Custom");
        let engine = TemplateEngine::new().with_custom_template(custom);
        let template = engine.template();
        assert_eq!(template.name, "my-template");
    }

    #[test]
    fn test_template_engine_render() {
        let engine = TemplateEngine::new();
        let adr = Adr::new(1, "Test");
        let config = Config::default();

        let output = engine.render(&adr, &config).unwrap();
        assert!(output.contains("# 1. Test"));
    }

    #[test]
    fn test_template_engine_render_custom() {
        let custom = Template::from_string("custom", "ADR {{ number }}: {{ title }}");
        let engine = TemplateEngine::new().with_custom_template(custom);
        let adr = Adr::new(42, "Custom ADR");
        let config = Config::default();

        let output = engine.render(&adr, &config).unwrap();
        assert_eq!(output, "ADR 42: Custom ADR");
    }

    // ========== Custom Template Tests ==========

    #[test]
    fn test_custom_template_all_fields() {
        let custom = Template::from_string(
            "full",
            r#"# {{ number }}. {{ title }}
Date: {{ date }}
Status: {{ status }}
Context: {{ context }}
Decision: {{ decision }}
Consequences: {{ consequences }}
Links: {% for link in links %}{{ link.kind }} {{ link.target }}{% endfor %}"#,
        );

        let mut adr = Adr::new(1, "Test");
        adr.status = AdrStatus::Accepted;
        adr.context = "Context text".into();
        adr.decision = "Decision text".into();
        adr.consequences = "Consequences text".into();
        adr.links.push(AdrLink::new(2, LinkKind::Amends));

        let config = Config::default();
        let output = custom.render(&adr, &config).unwrap();

        assert!(output.contains("# 1. Test"));
        assert!(output.contains("Status: Accepted"));
        assert!(output.contains("Context: Context text"));
        assert!(output.contains("Decision: Decision text"));
        assert!(output.contains("Consequences: Consequences text"));
        assert!(output.contains("Amends 2"));
    }

    #[test]
    fn test_custom_template_is_ng_flag() {
        let custom = Template::from_string(
            "ng-check",
            r#"{% if is_ng %}NextGen Mode{% else %}Compatible Mode{% endif %}"#,
        );

        let adr = Adr::new(1, "Test");

        let compat_config = Config::default();
        let output = custom.render(&adr, &compat_config).unwrap();
        assert_eq!(output, "Compatible Mode");

        let ng_config = Config {
            mode: ConfigMode::NextGen,
            ..Default::default()
        };
        let output = custom.render(&adr, &ng_config).unwrap();
        assert_eq!(output, "NextGen Mode");
    }

    #[test]
    fn test_custom_template_link_kinds() {
        let custom = Template::from_string(
            "links",
            r#"{% for link in links %}{{ link.kind }}|{% endfor %}"#,
        );

        let mut adr = Adr::new(1, "Test");
        adr.links.push(AdrLink::new(1, LinkKind::Supersedes));
        adr.links.push(AdrLink::new(2, LinkKind::SupersededBy));
        adr.links.push(AdrLink::new(3, LinkKind::Amends));
        adr.links.push(AdrLink::new(4, LinkKind::AmendedBy));
        adr.links.push(AdrLink::new(5, LinkKind::RelatesTo));
        adr.links
            .push(AdrLink::new(6, LinkKind::Custom("Depends on".into())));

        let config = Config::default();
        let output = custom.render(&adr, &config).unwrap();

        assert!(output.contains("Supersedes|"));
        assert!(output.contains("Superseded by|"));
        assert!(output.contains("Amends|"));
        assert!(output.contains("Amended by|"));
        assert!(output.contains("Relates to|"));
        assert!(output.contains("Depends on|"));
    }

    // ========== Error Cases ==========

    #[test]
    fn test_template_invalid_syntax() {
        let custom = Template::from_string("invalid", "{{ unclosed");
        let adr = Adr::new(1, "Test");
        let config = Config::default();

        let result = custom.render(&adr, &config);
        assert!(result.is_err());
    }

    #[test]
    fn test_template_undefined_variable() {
        let custom = Template::from_string("undefined", "{{ nonexistent }}");
        let adr = Adr::new(1, "Test");
        let config = Config::default();

        // minijinja treats undefined as empty string by default
        let result = custom.render(&adr, &config);
        assert!(result.is_ok());
    }

    // ========== Large Number Formatting ==========

    #[test]
    fn test_render_four_digit_number() {
        let template = Template::builtin(TemplateFormat::Nygard);
        let adr = Adr::new(9999, "Large Number");
        let config = Config::default();

        let output = template.render(&adr, &config).unwrap();
        assert!(output.contains("# 9999. Large Number"));
    }

    #[test]
    fn test_render_link_number_formatting() {
        let template = Template::builtin(TemplateFormat::Nygard);
        let mut adr = Adr::new(2, "Test");
        adr.links.push(AdrLink::new(1, LinkKind::Supersedes));

        let config = Config::default();
        let output = template.render(&adr, &config).unwrap();

        // Link should use 4-digit padding
        assert!(output.contains("0001-"));
    }
}
