//! Repository operations for managing ADRs.

use crate::{
    Adr, AdrLink, AdrStatus, Config, ConfigMode, Error, LinkKind, Parser, Result, Template,
    TemplateEngine, TemplateFormat, TemplateVariant,
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

        // Check if directory exists and count existing ADRs
        let existing_adrs = if adr_path.exists() {
            count_existing_adrs(&adr_path)
        } else {
            // Create the directory
            fs::create_dir_all(&adr_path)?;
            0
        };

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

        // Only create initial ADR if no ADRs exist
        if existing_adrs == 0 {
            let mut adr = Adr::new(1, "Record architecture decisions");
            adr.status = AdrStatus::Accepted;
            adr.context =
                "We need to record the architectural decisions made on this project.".into();
            adr.decision = "We will use Architecture Decision Records, as described by Michael Nygard in his article \"Documenting Architecture Decisions\".".into();
            adr.consequences = "See Michael Nygard's article, linked above. For a lightweight ADR toolset, see Nat Pryce's adr-tools.".into();
            repo.create(&adr)?;
        }

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

    /// Set the template variant.
    pub fn with_template_variant(mut self, variant: TemplateVariant) -> Self {
        self.template_engine = self.template_engine.with_variant(variant);
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

    /// Change the status of an ADR.
    ///
    /// If the new status is `Superseded` and `superseded_by` is provided,
    /// a superseded-by link will be added automatically.
    pub fn set_status(
        &self,
        number: u32,
        status: AdrStatus,
        superseded_by: Option<u32>,
    ) -> Result<PathBuf> {
        let mut adr = self.get(number)?;
        adr.status = status.clone();

        // If superseded by another ADR, add the link
        if let (AdrStatus::Superseded, Some(by)) = (&status, superseded_by) {
            // Check that the superseding ADR exists
            let _ = self.get(by)?;

            // Add superseded-by link if not already present
            if !adr
                .links
                .iter()
                .any(|l| matches!(l.kind, LinkKind::SupersededBy) && l.target == by)
            {
                adr.add_link(AdrLink::new(by, LinkKind::SupersededBy));
            }
        }

        self.update(&adr)
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

/// Count existing ADR files in a directory.
fn count_existing_adrs(path: &Path) -> usize {
    if !path.is_dir() {
        return 0;
    }

    fs::read_dir(path)
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .filter(|e| {
                    let path = e.path();
                    path.is_file()
                        && path.extension().is_some_and(|ext| ext == "md")
                        && path.file_name().and_then(|n| n.to_str()).is_some_and(|n| {
                            // Match NNNN-*.md pattern (adr-tools style)
                            n.len() > 5 && n[..4].chars().all(|c| c.is_ascii_digit())
                        })
                })
                .count()
        })
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // ========== Initialization Tests ==========

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
    fn test_init_repository_custom_dir() {
        let temp = TempDir::new().unwrap();
        let repo = Repository::init(temp.path(), Some("decisions".into()), false).unwrap();

        assert!(temp.path().join("decisions").exists());
        assert_eq!(repo.config().adr_dir, PathBuf::from("decisions"));
    }

    #[test]
    fn test_init_repository_nested_dir() {
        let temp = TempDir::new().unwrap();
        let _repo =
            Repository::init(temp.path(), Some("docs/architecture/adr".into()), false).unwrap();

        assert!(temp.path().join("docs/architecture/adr").exists());
    }

    #[test]
    fn test_init_repository_already_exists_skips_initial_adr() {
        let temp = TempDir::new().unwrap();
        Repository::init(temp.path(), None, false).unwrap();

        // Re-init should succeed but not create another ADR
        let repo = Repository::init(temp.path(), None, false).unwrap();
        let adrs = repo.list().unwrap();
        assert_eq!(adrs.len(), 1); // Still just the original initial ADR
    }

    #[test]
    fn test_init_with_existing_adrs_skips_initial() {
        let temp = TempDir::new().unwrap();
        let adr_dir = temp.path().join("doc/adr");
        fs::create_dir_all(&adr_dir).unwrap();

        // Create some existing ADR files
        fs::write(
            adr_dir.join("0001-existing-decision.md"),
            "# 1. Existing Decision\n\nDate: 2024-01-01\n\n## Status\n\nAccepted\n\n## Context\n\nTest\n\n## Decision\n\nTest\n\n## Consequences\n\nTest\n",
        )
        .unwrap();
        fs::write(
            adr_dir.join("0002-another-decision.md"),
            "# 2. Another Decision\n\nDate: 2024-01-02\n\n## Status\n\nAccepted\n\n## Context\n\nTest\n\n## Decision\n\nTest\n\n## Consequences\n\nTest\n",
        )
        .unwrap();

        // Init should succeed and NOT create initial ADR
        let repo = Repository::init(temp.path(), None, false).unwrap();
        let adrs = repo.list().unwrap();
        assert_eq!(adrs.len(), 2); // Only the existing ADRs, no "Record architecture decisions"
        assert_eq!(adrs[0].title, "Existing Decision");
        assert_eq!(adrs[1].title, "Another Decision");
    }

    #[test]
    fn test_init_creates_first_adr() {
        let temp = TempDir::new().unwrap();
        let repo = Repository::init(temp.path(), None, false).unwrap();

        let adr = repo.get(1).unwrap();
        assert_eq!(adr.title, "Record architecture decisions");
        assert_eq!(adr.status, AdrStatus::Accepted);
        assert!(!adr.context.is_empty());
        assert!(!adr.decision.is_empty());
        assert!(!adr.consequences.is_empty());
    }

    // ========== Open Tests ==========

    #[test]
    fn test_open_repository() {
        let temp = TempDir::new().unwrap();
        Repository::init(temp.path(), None, false).unwrap();

        let repo = Repository::open(temp.path()).unwrap();
        assert_eq!(repo.list().unwrap().len(), 1);
    }

    #[test]
    fn test_open_repository_not_found() {
        let temp = TempDir::new().unwrap();
        let result = Repository::open(temp.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_open_or_default() {
        let temp = TempDir::new().unwrap();
        let repo = Repository::open_or_default(temp.path());
        assert_eq!(repo.config().adr_dir, PathBuf::from("doc/adr"));
    }

    #[test]
    fn test_open_or_default_existing() {
        let temp = TempDir::new().unwrap();
        Repository::init(temp.path(), Some("custom".into()), false).unwrap();

        let repo = Repository::open_or_default(temp.path());
        assert_eq!(repo.config().adr_dir, PathBuf::from("custom"));
    }

    // ========== Create and List Tests ==========

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
    fn test_create_multiple() {
        let temp = TempDir::new().unwrap();
        let repo = Repository::init(temp.path(), None, false).unwrap();

        repo.new_adr("Second").unwrap();
        repo.new_adr("Third").unwrap();
        repo.new_adr("Fourth").unwrap();

        let adrs = repo.list().unwrap();
        assert_eq!(adrs.len(), 4);
        assert_eq!(adrs[0].number, 1);
        assert_eq!(adrs[1].number, 2);
        assert_eq!(adrs[2].number, 3);
        assert_eq!(adrs[3].number, 4);
    }

    #[test]
    fn test_list_sorted_by_number() {
        let temp = TempDir::new().unwrap();
        let repo = Repository::init(temp.path(), None, false).unwrap();

        repo.new_adr("B").unwrap();
        repo.new_adr("A").unwrap();
        repo.new_adr("C").unwrap();

        let adrs = repo.list().unwrap();
        assert!(adrs.windows(2).all(|w| w[0].number < w[1].number));
    }

    #[test]
    fn test_next_number() {
        let temp = TempDir::new().unwrap();
        let repo = Repository::init(temp.path(), None, false).unwrap();

        assert_eq!(repo.next_number().unwrap(), 2);

        repo.new_adr("Second").unwrap();
        assert_eq!(repo.next_number().unwrap(), 3);
    }

    #[test]
    fn test_create_file_exists() {
        let temp = TempDir::new().unwrap();
        let repo = Repository::init(temp.path(), None, false).unwrap();

        let (_, path) = repo.new_adr("Test ADR").unwrap();
        assert!(path.exists());
        assert!(path.to_string_lossy().contains("0002-test-adr.md"));
    }

    // ========== Get and Find Tests ==========

    #[test]
    fn test_get_by_number() {
        let temp = TempDir::new().unwrap();
        let repo = Repository::init(temp.path(), None, false).unwrap();
        repo.new_adr("Second").unwrap();

        let adr = repo.get(2).unwrap();
        assert_eq!(adr.title, "Second");
    }

    #[test]
    fn test_get_not_found() {
        let temp = TempDir::new().unwrap();
        let repo = Repository::init(temp.path(), None, false).unwrap();

        let result = repo.get(99);
        assert!(result.is_err());
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
    fn test_find_fuzzy_match() {
        let temp = TempDir::new().unwrap();
        let repo = Repository::init(temp.path(), None, false).unwrap();
        repo.new_adr("Use PostgreSQL for database").unwrap();
        repo.new_adr("Use Redis for caching").unwrap();

        let adr = repo.find("postgres").unwrap();
        assert!(adr.title.contains("PostgreSQL"));
    }

    #[test]
    fn test_find_not_found() {
        let temp = TempDir::new().unwrap();
        let repo = Repository::init(temp.path(), None, false).unwrap();

        let result = repo.find("nonexistent");
        assert!(result.is_err());
    }

    // ========== Supersede Tests ==========

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

    #[test]
    fn test_supersede_creates_bidirectional_links() {
        let temp = TempDir::new().unwrap();
        let repo = Repository::init(temp.path(), None, false).unwrap();

        repo.supersede("New approach", 1).unwrap();

        let old_adr = repo.get(1).unwrap();
        assert_eq!(old_adr.links.len(), 1);
        assert_eq!(old_adr.links[0].target, 2);
        assert_eq!(old_adr.links[0].kind, LinkKind::SupersededBy);

        let new_adr = repo.get(2).unwrap();
        assert_eq!(new_adr.links.len(), 1);
        assert_eq!(new_adr.links[0].target, 1);
        assert_eq!(new_adr.links[0].kind, LinkKind::Supersedes);
    }

    #[test]
    fn test_supersede_not_found() {
        let temp = TempDir::new().unwrap();
        let repo = Repository::init(temp.path(), None, false).unwrap();

        let result = repo.supersede("New", 99);
        assert!(result.is_err());
    }

    // ========== Set Status Tests ==========

    #[test]
    fn test_set_status_accepted() {
        let temp = TempDir::new().unwrap();
        let repo = Repository::init(temp.path(), None, false).unwrap();
        repo.new_adr("Test Decision").unwrap();

        repo.set_status(2, AdrStatus::Accepted, None).unwrap();

        let adr = repo.get(2).unwrap();
        assert_eq!(adr.status, AdrStatus::Accepted);
    }

    #[test]
    fn test_set_status_deprecated() {
        let temp = TempDir::new().unwrap();
        let repo = Repository::init(temp.path(), None, false).unwrap();
        repo.new_adr("Old Decision").unwrap();

        repo.set_status(2, AdrStatus::Deprecated, None).unwrap();

        let adr = repo.get(2).unwrap();
        assert_eq!(adr.status, AdrStatus::Deprecated);
    }

    #[test]
    fn test_set_status_superseded_with_link() {
        let temp = TempDir::new().unwrap();
        let repo = Repository::init(temp.path(), None, false).unwrap();
        repo.new_adr("First Decision").unwrap();
        repo.new_adr("Second Decision").unwrap();

        repo.set_status(2, AdrStatus::Superseded, Some(3)).unwrap();

        let adr = repo.get(2).unwrap();
        assert_eq!(adr.status, AdrStatus::Superseded);
        assert_eq!(adr.links.len(), 1);
        assert_eq!(adr.links[0].target, 3);
        assert_eq!(adr.links[0].kind, LinkKind::SupersededBy);
    }

    #[test]
    fn test_set_status_superseded_without_link() {
        let temp = TempDir::new().unwrap();
        let repo = Repository::init(temp.path(), None, false).unwrap();
        repo.new_adr("Decision").unwrap();

        repo.set_status(2, AdrStatus::Superseded, None).unwrap();

        let adr = repo.get(2).unwrap();
        assert_eq!(adr.status, AdrStatus::Superseded);
        assert_eq!(adr.links.len(), 0);
    }

    #[test]
    fn test_set_status_custom() {
        let temp = TempDir::new().unwrap();
        let repo = Repository::init(temp.path(), None, false).unwrap();
        repo.new_adr("Test Decision").unwrap();

        repo.set_status(2, AdrStatus::Custom("Draft".into()), None)
            .unwrap();

        let adr = repo.get(2).unwrap();
        assert_eq!(adr.status, AdrStatus::Custom("Draft".into()));
    }

    #[test]
    fn test_set_status_adr_not_found() {
        let temp = TempDir::new().unwrap();
        let repo = Repository::init(temp.path(), None, false).unwrap();

        let result = repo.set_status(99, AdrStatus::Accepted, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_set_status_superseded_by_not_found() {
        let temp = TempDir::new().unwrap();
        let repo = Repository::init(temp.path(), None, false).unwrap();
        repo.new_adr("Decision").unwrap();

        let result = repo.set_status(2, AdrStatus::Superseded, Some(99));
        assert!(result.is_err());
    }

    // ========== Link Tests ==========

    #[test]
    fn test_link_adrs() {
        let temp = TempDir::new().unwrap();
        let repo = Repository::init(temp.path(), None, false).unwrap();
        repo.new_adr("Second").unwrap();

        repo.link(1, 2, LinkKind::Amends, LinkKind::AmendedBy)
            .unwrap();

        let adr1 = repo.get(1).unwrap();
        assert_eq!(adr1.links.len(), 1);
        assert_eq!(adr1.links[0].target, 2);
        assert_eq!(adr1.links[0].kind, LinkKind::Amends);

        let adr2 = repo.get(2).unwrap();
        assert_eq!(adr2.links.len(), 1);
        assert_eq!(adr2.links[0].target, 1);
        assert_eq!(adr2.links[0].kind, LinkKind::AmendedBy);
    }

    #[test]
    fn test_link_relates_to() {
        let temp = TempDir::new().unwrap();
        let repo = Repository::init(temp.path(), None, false).unwrap();
        repo.new_adr("Second").unwrap();

        repo.link(1, 2, LinkKind::RelatesTo, LinkKind::RelatesTo)
            .unwrap();

        let adr1 = repo.get(1).unwrap();
        assert_eq!(adr1.links[0].kind, LinkKind::RelatesTo);

        let adr2 = repo.get(2).unwrap();
        assert_eq!(adr2.links[0].kind, LinkKind::RelatesTo);
    }

    // ========== Update Tests ==========

    #[test]
    fn test_update_adr() {
        let temp = TempDir::new().unwrap();
        let repo = Repository::init(temp.path(), None, false).unwrap();

        let mut adr = repo.get(1).unwrap();
        adr.status = AdrStatus::Deprecated;

        repo.update(&adr).unwrap();

        let updated = repo.get(1).unwrap();
        assert_eq!(updated.status, AdrStatus::Deprecated);
    }

    #[test]
    fn test_update_preserves_content() {
        let temp = TempDir::new().unwrap();
        let repo = Repository::init(temp.path(), None, false).unwrap();

        let mut adr = repo.get(1).unwrap();
        let original_title = adr.title.clone();
        adr.status = AdrStatus::Deprecated;

        repo.update(&adr).unwrap();

        let updated = repo.get(1).unwrap();
        assert_eq!(updated.title, original_title);
    }

    // ========== Read/Write Content Tests ==========

    #[test]
    fn test_read_content() {
        let temp = TempDir::new().unwrap();
        let repo = Repository::init(temp.path(), None, false).unwrap();

        let adr = repo.get(1).unwrap();
        let content = repo.read_content(&adr).unwrap();

        assert!(content.contains("Record architecture decisions"));
        assert!(content.contains("## Status"));
    }

    #[test]
    fn test_write_content() {
        let temp = TempDir::new().unwrap();
        let repo = Repository::init(temp.path(), None, false).unwrap();

        let adr = repo.get(1).unwrap();
        let new_content = "# 1. Modified\n\n## Status\n\nAccepted\n";

        repo.write_content(&adr, new_content).unwrap();

        let content = repo.read_content(&adr).unwrap();
        assert!(content.contains("Modified"));
    }

    // ========== Template Configuration Tests ==========

    #[test]
    fn test_with_template_format() {
        let temp = TempDir::new().unwrap();
        let repo = Repository::init(temp.path(), None, false)
            .unwrap()
            .with_template_format(TemplateFormat::Madr);

        let (_, path) = repo.new_adr("MADR Test").unwrap();
        let content = fs::read_to_string(path).unwrap();

        assert!(content.contains("Context and Problem Statement"));
    }

    #[test]
    fn test_with_custom_template() {
        let temp = TempDir::new().unwrap();
        let custom = Template::from_string("custom", "# ADR {{ number }}: {{ title }}");
        let repo = Repository::init(temp.path(), None, false)
            .unwrap()
            .with_custom_template(custom);

        let (_, path) = repo.new_adr("Custom Test").unwrap();
        let content = fs::read_to_string(path).unwrap();

        assert_eq!(content, "# ADR 2: Custom Test");
    }

    // ========== Accessor Tests ==========

    #[test]
    fn test_root() {
        let temp = TempDir::new().unwrap();
        let repo = Repository::init(temp.path(), None, false).unwrap();

        assert_eq!(repo.root(), temp.path());
    }

    #[test]
    fn test_config() {
        let temp = TempDir::new().unwrap();
        let repo = Repository::init(temp.path(), Some("custom".into()), true).unwrap();

        assert_eq!(repo.config().adr_dir, PathBuf::from("custom"));
        assert!(repo.config().is_next_gen());
    }

    #[test]
    fn test_adr_path() {
        let temp = TempDir::new().unwrap();
        let repo = Repository::init(temp.path(), Some("my/adrs".into()), false).unwrap();

        assert_eq!(repo.adr_path(), temp.path().join("my/adrs"));
    }

    // ========== NextGen Mode Tests ==========

    #[test]
    fn test_ng_mode_creates_frontmatter() {
        let temp = TempDir::new().unwrap();
        let repo = Repository::init(temp.path(), None, true).unwrap();

        let (_, path) = repo.new_adr("NG Test").unwrap();
        let content = fs::read_to_string(path).unwrap();

        assert!(content.starts_with("---"));
        assert!(content.contains("number: 2"));
        assert!(content.contains("title: NG Test"));
    }

    #[test]
    fn test_ng_mode_parses_frontmatter() {
        let temp = TempDir::new().unwrap();
        let repo = Repository::init(temp.path(), None, true).unwrap();

        repo.new_adr("NG ADR").unwrap();

        let adr = repo.get(2).unwrap();
        assert_eq!(adr.title, "NG ADR");
        assert_eq!(adr.number, 2);
    }

    // ========== Edge Cases ==========

    #[test]
    fn test_list_empty_after_init_removal() {
        let temp = TempDir::new().unwrap();
        let repo = Repository::init(temp.path(), None, false).unwrap();

        // Remove the initial ADR
        fs::remove_file(
            repo.adr_path()
                .join("0001-record-architecture-decisions.md"),
        )
        .unwrap();

        let adrs = repo.list().unwrap();
        assert!(adrs.is_empty());
    }

    #[test]
    fn test_list_ignores_non_adr_files() {
        let temp = TempDir::new().unwrap();
        let repo = Repository::init(temp.path(), None, false).unwrap();

        // Create non-ADR files
        fs::write(repo.adr_path().join("README.md"), "# README").unwrap();
        fs::write(repo.adr_path().join("notes.txt"), "Notes").unwrap();

        let adrs = repo.list().unwrap();
        assert_eq!(adrs.len(), 1); // Only the initial ADR
    }

    #[test]
    fn test_special_characters_in_title() {
        let temp = TempDir::new().unwrap();
        let repo = Repository::init(temp.path(), None, false).unwrap();

        let (adr, path) = repo.new_adr("Use C++ & Rust!").unwrap();
        assert!(path.exists());
        assert_eq!(adr.title, "Use C++ & Rust!");
    }
}
