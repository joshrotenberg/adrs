//! Repository operations for managing ADRs.

use crate::{
    Adr, AdrLink, AdrStatus, Config, ConfigMode, Error, LinkKind, Parser, Result, Template,
    TemplateEngine, TemplateFormat,
};
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// A repository of Architecture Decision Records.
#[derive(Debug)]
pub struct Repository {
    /// The root directory of the project.
    root: PathBuf,

    /// Configuration for this repository.
    config: Config,

    /// Parser for reading ADRs.
    parser: Parser,

    /// Template engine for creating ADRs.
    template_engine: TemplateEngine,
}

impl Repository {
    /// Open an existing repository at the given root.
    pub fn open(root: impl Into<PathBuf>) -> Result<Self> {
        let root = root.into();
        let config = Config::load(&root)?;

        Ok(Self {
            root,
            config,
            parser: Parser::new(),
            template_engine: TemplateEngine::new(),
        })
    }

    /// Open a repository, or create default config if not found.
    pub fn open_or_default(root: impl Into<PathBuf>) -> Self {
        let root = root.into();
        let config = Config::load_or_default(&root);

        Self {
            root,
            config,
            parser: Parser::new(),
            template_engine: TemplateEngine::new(),
        }
    }

    /// Initialize a new repository at the given root.
    pub fn init(root: impl Into<PathBuf>, adr_dir: Option<PathBuf>, ng: bool) -> Result<Self> {
        let root = root.into();
        let adr_dir = adr_dir.unwrap_or_else(|| PathBuf::from(crate::config::DEFAULT_ADR_DIR));
        let adr_path = root.join(&adr_dir);

        // Check if already initialized
        if adr_path.exists() {
            return Err(Error::AdrDirExists(adr_path));
        }

        // Create the directory
        fs::create_dir_all(&adr_path)?;

        // Create config
        let config = Config {
            adr_dir,
            mode: if ng {
                ConfigMode::NextGen
            } else {
                ConfigMode::Compatible
            },
            ..Default::default()
        };
        config.save(&root)?;

        let repo = Self {
            root,
            config,
            parser: Parser::new(),
            template_engine: TemplateEngine::new(),
        };

        // Create the initial ADR
        let mut adr = Adr::new(1, "Record architecture decisions");
        adr.status = AdrStatus::Accepted;
        adr.context = "We need to record the architectural decisions made on this project.".into();
        adr.decision = "We will use Architecture Decision Records, as described by Michael Nygard in his article \"Documenting Architecture Decisions\".".into();
        adr.consequences = "See Michael Nygard's article, linked above. For a lightweight ADR toolset, see Nat Pryce's adr-tools.".into();
        repo.create(&adr)?;

        Ok(repo)
    }

    /// Get the repository root path.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Get the configuration.
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Get the full path to the ADR directory.
    pub fn adr_path(&self) -> PathBuf {
        self.config.adr_path(&self.root)
    }

    /// Set the template format.
    pub fn with_template_format(mut self, format: TemplateFormat) -> Self {
        self.template_engine = self.template_engine.with_format(format);
        self
    }

    /// Set a custom template.
    pub fn with_custom_template(mut self, template: Template) -> Self {
        self.template_engine = self.template_engine.with_custom_template(template);
        self
    }

    /// List all ADRs in the repository.
    pub fn list(&self) -> Result<Vec<Adr>> {
        let adr_path = self.adr_path();
        if !adr_path.exists() {
            return Err(Error::AdrDirNotFound);
        }

        let mut adrs: Vec<Adr> = WalkDir::new(&adr_path)
            .max_depth(1)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path().extension().is_some_and(|ext| ext == "md")
                    && e.path()
                        .file_name()
                        .and_then(|n| n.to_str())
                        .is_some_and(|n| n.chars().next().is_some_and(|c| c.is_ascii_digit()))
            })
            .filter_map(|e| self.parser.parse_file(e.path()).ok())
            .collect();

        adrs.sort_by_key(|a| a.number);
        Ok(adrs)
    }

    /// Get the next available ADR number.
    pub fn next_number(&self) -> Result<u32> {
        let adrs = self.list()?;
        Ok(adrs.last().map(|a| a.number + 1).unwrap_or(1))
    }

    /// Find an ADR by number.
    pub fn get(&self, number: u32) -> Result<Adr> {
        let adrs = self.list()?;
        adrs.into_iter()
            .find(|a| a.number == number)
            .ok_or_else(|| Error::AdrNotFound(number.to_string()))
    }

    /// Find an ADR by query (number or fuzzy title match).
    pub fn find(&self, query: &str) -> Result<Adr> {
        // Try parsing as number first
        if let Ok(number) = query.parse::<u32>() {
            return self.get(number);
        }

        // Fuzzy match on title
        let adrs = self.list()?;
        let matcher = SkimMatcherV2::default();

        let mut matches: Vec<_> = adrs
            .into_iter()
            .filter_map(|adr| {
                let score = matcher.fuzzy_match(&adr.title, query)?;
                Some((adr, score))
            })
            .collect();

        matches.sort_by(|a, b| b.1.cmp(&a.1));

        match matches.len() {
            0 => Err(Error::AdrNotFound(query.to_string())),
            1 => Ok(matches.remove(0).0),
            _ => {
                // If top match is significantly better, use it
                if matches[0].1 > matches[1].1 * 2 {
                    Ok(matches.remove(0).0)
                } else {
                    Err(Error::AmbiguousAdr {
                        query: query.to_string(),
                        matches: matches
                            .iter()
                            .take(5)
                            .map(|(a, _)| a.title.clone())
                            .collect(),
                    })
                }
            }
        }
    }

    /// Create a new ADR.
    pub fn create(&self, adr: &Adr) -> Result<PathBuf> {
        let path = self.adr_path().join(adr.filename());

        let content = self.template_engine.render(adr, &self.config)?;
        fs::write(&path, content)?;

        Ok(path)
    }

    /// Create a new ADR with the given title.
    pub fn new_adr(&self, title: impl Into<String>) -> Result<(Adr, PathBuf)> {
        let number = self.next_number()?;
        let adr = Adr::new(number, title);
        let path = self.create(&adr)?;
        Ok((adr, path))
    }

    /// Create a new ADR that supersedes another.
    pub fn supersede(&self, title: impl Into<String>, superseded: u32) -> Result<(Adr, PathBuf)> {
        let number = self.next_number()?;
        let mut adr = Adr::new(number, title);
        adr.add_link(AdrLink::new(superseded, LinkKind::Supersedes));

        // Update the superseded ADR
        let mut old_adr = self.get(superseded)?;
        old_adr.status = AdrStatus::Superseded;
        old_adr.add_link(AdrLink::new(number, LinkKind::SupersededBy));
        self.update(&old_adr)?;

        let path = self.create(&adr)?;
        Ok((adr, path))
    }

    /// Link two ADRs together.
    pub fn link(
        &self,
        source: u32,
        target: u32,
        source_kind: LinkKind,
        target_kind: LinkKind,
    ) -> Result<()> {
        let mut source_adr = self.get(source)?;
        let mut target_adr = self.get(target)?;

        source_adr.add_link(AdrLink::new(target, source_kind));
        target_adr.add_link(AdrLink::new(source, target_kind));

        self.update(&source_adr)?;
        self.update(&target_adr)?;

        Ok(())
    }

    /// Update an existing ADR.
    pub fn update(&self, adr: &Adr) -> Result<PathBuf> {
        let path = adr
            .path
            .clone()
            .unwrap_or_else(|| self.adr_path().join(adr.filename()));

        let content = self.template_engine.render(adr, &self.config)?;
        fs::write(&path, content)?;

        Ok(path)
    }

    /// Read the content of an ADR file.
    pub fn read_content(&self, adr: &Adr) -> Result<String> {
        let path = adr
            .path
            .as_ref()
            .cloned()
            .unwrap_or_else(|| self.adr_path().join(adr.filename()));

        Ok(fs::read_to_string(path)?)
    }

    /// Write content to an ADR file.
    pub fn write_content(&self, adr: &Adr, content: &str) -> Result<PathBuf> {
        let path = adr
            .path
            .as_ref()
            .cloned()
            .unwrap_or_else(|| self.adr_path().join(adr.filename()));

        fs::write(&path, content)?;
        Ok(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_init_repository() {
        let temp = TempDir::new().unwrap();
        let repo = Repository::init(temp.path(), None, false).unwrap();

        assert!(repo.adr_path().exists());
        assert!(temp.path().join(".adr-dir").exists());

        let adrs = repo.list().unwrap();
        assert_eq!(adrs.len(), 1);
        assert_eq!(adrs[0].number, 1);
        assert_eq!(adrs[0].title, "Record architecture decisions");
    }

    #[test]
    fn test_init_repository_ng() {
        let temp = TempDir::new().unwrap();
        let repo = Repository::init(temp.path(), None, true).unwrap();

        assert!(temp.path().join("adrs.toml").exists());
        assert!(repo.config().is_next_gen());
    }

    #[test]
    fn test_create_and_list() {
        let temp = TempDir::new().unwrap();
        let repo = Repository::init(temp.path(), None, false).unwrap();

        let (adr, _) = repo.new_adr("Use Rust").unwrap();
        assert_eq!(adr.number, 2);

        let adrs = repo.list().unwrap();
        assert_eq!(adrs.len(), 2);
    }

    #[test]
    fn test_find_by_number() {
        let temp = TempDir::new().unwrap();
        let repo = Repository::init(temp.path(), None, false).unwrap();

        let adr = repo.find("1").unwrap();
        assert_eq!(adr.number, 1);
    }

    #[test]
    fn test_find_by_title() {
        let temp = TempDir::new().unwrap();
        let repo = Repository::init(temp.path(), None, false).unwrap();

        let adr = repo.find("architecture").unwrap();
        assert_eq!(adr.number, 1);
    }

    #[test]
    fn test_supersede() {
        let temp = TempDir::new().unwrap();
        let repo = Repository::init(temp.path(), None, false).unwrap();

        let (new_adr, _) = repo.supersede("New approach", 1).unwrap();
        assert_eq!(new_adr.number, 2);
        assert_eq!(new_adr.links.len(), 1);
        assert_eq!(new_adr.links[0].kind, LinkKind::Supersedes);

        let old_adr = repo.get(1).unwrap();
        assert_eq!(old_adr.status, AdrStatus::Superseded);
    }
}
