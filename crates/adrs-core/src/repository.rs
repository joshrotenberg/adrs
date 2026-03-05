//! # Repository Operations
//!
//! The [`Repository`] struct is the main entry point for ADR operations.
//!
//! ## Overview
//!
//! A repository represents a collection of ADRs stored in a directory.
//! It provides methods for:
//!
//! | Category | Methods |
//! |----------|---------|
//! | **Lifecycle** | [`open`](Repository::open), [`init`](Repository::init) |
//! | **Query** | [`list`](Repository::list), [`get`](Repository::get), [`find`](Repository::find) |
//! | **Mutation** | [`new_adr`](Repository::new_adr), [`supersede`](Repository::supersede), [`link`](Repository::link) |
//!
//! ## Quick Start
//!
//! ```rust
//! # use adrs_core::doctest_helpers::temp_repo;
//! # let (_temp, repo) = temp_repo().unwrap();
//! // List all ADRs
//! let adrs = repo.list().unwrap();
//! println!("Found {} ADRs", adrs.len());
//!
//! // Create a new ADR
//! let (adr, path) = repo.new_adr("Use PostgreSQL for persistence").unwrap();
//! println!("Created ADR #{} at {}", adr.number, path.display());
//! ```

use crate::{
    Adr, AdrLink, AdrStatus, Config, ConfigMode, Error, LinkKind, Parser, Result, Template,
    TemplateEngine, TemplateFormat, TemplateVariant,
};
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use walkdir::WalkDir;

/// Regex for matching the status line in YAML frontmatter.
static FM_STATUS_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?m)^status:\s*.*$").unwrap());

/// Regex for matching the links block in YAML frontmatter (multi-line).
static FM_LINKS_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^links:\n(?:(?:  .+\n)*)").unwrap());

/// Regex for matching the tags block in YAML frontmatter (multi-line).
static FM_TAGS_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^tags:\n(?:(?:  .+\n)*)").unwrap());

/// A repository of Architecture Decision Records.
///
/// The `Repository` is the main entry point for all ADR operations.
/// It manages ADR files in a directory and provides methods for
/// creating, reading, updating, and querying ADRs.
///
/// # Creating a Repository
///
/// ```rust
/// # use adrs_core::doctest_helpers::temp_repo;
/// // Open existing or create new
/// let (_temp, repo) = temp_repo().unwrap();
///
/// // The repository has one ADR created by init
/// assert_eq!(repo.list().unwrap().len(), 1);
/// ```
///
/// # Thread Safety
///
/// `Repository` is `Send` but not `Sync`. Each thread should have
/// its own `Repository` instance. File operations are not internally
/// locked; use external coordination if multiple processes access
/// the same repository.
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
    /// Opens an existing repository at the given root.
    ///
    /// Searches for `.adr-dir` (Compatible mode) or `adrs.toml` (NextGen mode)
    /// in the root directory to locate the ADR configuration.
    ///
    /// # Errors
    ///
    /// Returns [`Error::AdrDirNotFound`] if no configuration file is found.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use adrs_core::Repository;
    /// use tempfile::TempDir;
    ///
    /// let temp = TempDir::new().unwrap();
    ///
    /// // First initialize a repository
    /// let repo = Repository::init(temp.path(), None, false).unwrap();
    /// drop(repo);
    ///
    /// // Now open it
    /// let repo = Repository::open(temp.path()).unwrap();
    /// assert_eq!(repo.list().unwrap().len(), 1);
    /// ```
    pub fn open(root: impl Into<PathBuf>) -> Result<Self> {
        let root = root.into();
        let config = Config::load(&root)?;
        let template_engine = Self::engine_from_config(&config);

        Ok(Self {
            root,
            config,
            parser: Parser::new(),
            template_engine,
        })
    }

    /// Opens a repository, or uses default configuration if not found.
    ///
    /// Unlike [`open`](Self::open), this method never fails. If no
    /// configuration file exists, it uses default settings without
    /// creating any files.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use adrs_core::Repository;
    /// use tempfile::TempDir;
    ///
    /// let temp = TempDir::new().unwrap();
    ///
    /// // Works even without initialization
    /// let repo = Repository::open_or_default(temp.path());
    /// assert_eq!(repo.config().adr_dir.to_str(), Some("doc/adr"));
    /// ```
    pub fn open_or_default(root: impl Into<PathBuf>) -> Self {
        let root = root.into();
        let config = Config::load_or_default(&root);
        let template_engine = Self::engine_from_config(&config);

        Self {
            root,
            config,
            parser: Parser::new(),
            template_engine,
        }
    }

    /// Initializes a new repository at the given root.
    ///
    /// Creates the ADR directory and configuration file, then creates
    /// an initial ADR documenting the decision to use ADRs.
    ///
    /// # Arguments
    ///
    /// * `root` - The project root directory
    /// * `adr_dir` - Custom ADR directory (default: `doc/adr`)
    /// * `ng` - If true, use NextGen mode with `adrs.toml`; otherwise Compatible mode
    ///
    /// # Examples
    ///
    /// ```rust
    /// use adrs_core::Repository;
    /// use tempfile::TempDir;
    ///
    /// let temp = TempDir::new().unwrap();
    /// let repo = Repository::init(temp.path(), None, false).unwrap();
    ///
    /// // Init creates the first ADR automatically
    /// let adrs = repo.list().unwrap();
    /// assert_eq!(adrs.len(), 1);
    /// assert_eq!(adrs[0].title, "Record architecture decisions");
    /// ```
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

        let template_engine = Self::engine_from_config(&config);

        let repo = Self {
            root,
            config,
            parser: Parser::new(),
            template_engine,
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

    /// Returns the repository root path.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use adrs_core::doctest_helpers::temp_repo;
    /// let (temp, repo) = temp_repo().unwrap();
    /// assert_eq!(repo.root(), temp.path());
    /// ```
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Returns the repository configuration.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use adrs_core::doctest_helpers::temp_repo_nextgen;
    /// let (_temp, repo) = temp_repo_nextgen().unwrap();
    /// assert!(repo.config().is_next_gen());
    /// ```
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Returns the full path to the ADR directory.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use adrs_core::doctest_helpers::temp_repo;
    /// let (temp, repo) = temp_repo().unwrap();
    /// assert_eq!(repo.adr_path(), temp.path().join("doc/adr"));
    /// ```
    pub fn adr_path(&self) -> PathBuf {
        self.config.adr_path(&self.root)
    }

    /// Build a template engine that respects the config's template format.
    fn engine_from_config(config: &Config) -> TemplateEngine {
        let mut engine = TemplateEngine::new();
        if let Some(ref fmt) = config.templates.format
            && let Ok(format) = fmt.parse::<TemplateFormat>()
        {
            engine = engine.with_format(format);
        }
        engine
    }

    /// Sets the template format for new ADRs.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use adrs_core::{Repository, TemplateFormat};
    /// use tempfile::TempDir;
    ///
    /// let temp = TempDir::new().unwrap();
    /// let repo = Repository::init(temp.path(), None, false)
    ///     .unwrap()
    ///     .with_template_format(TemplateFormat::Madr);
    ///
    /// let (_, path) = repo.new_adr("Use MADR format").unwrap();
    /// let content = std::fs::read_to_string(path).unwrap();
    /// assert!(content.contains("Context and Problem Statement"));
    /// ```
    pub fn with_template_format(mut self, format: TemplateFormat) -> Self {
        self.template_engine = self.template_engine.with_format(format);
        self
    }

    /// Sets the template variant (full, minimal, or bare).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use adrs_core::{Repository, TemplateVariant};
    /// use tempfile::TempDir;
    ///
    /// let temp = TempDir::new().unwrap();
    /// let repo = Repository::init(temp.path(), None, false)
    ///     .unwrap()
    ///     .with_template_variant(TemplateVariant::Minimal);
    ///
    /// // Minimal variant has essential sections only
    /// let (_, path) = repo.new_adr("Quick decision").unwrap();
    /// assert!(path.exists());
    /// ```
    pub fn with_template_variant(mut self, variant: TemplateVariant) -> Self {
        self.template_engine = self.template_engine.with_variant(variant);
        self
    }

    /// Sets a custom template for new ADRs.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use adrs_core::{Repository, Template};
    /// use tempfile::TempDir;
    ///
    /// let temp = TempDir::new().unwrap();
    /// let custom = Template::from_string("custom", "# ADR {{ number }}: {{ title }}");
    /// let repo = Repository::init(temp.path(), None, false)
    ///     .unwrap()
    ///     .with_custom_template(custom);
    ///
    /// let (_, path) = repo.new_adr("Custom format").unwrap();
    /// let content = std::fs::read_to_string(path).unwrap();
    /// assert_eq!(content, "# ADR 2: Custom format");
    /// ```
    pub fn with_custom_template(mut self, template: Template) -> Self {
        self.template_engine = self.template_engine.with_custom_template(template);
        self
    }

    /// Lists all ADRs in the repository.
    ///
    /// Returns ADRs sorted by number in ascending order.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use adrs_core::doctest_helpers::temp_repo;
    /// let (_temp, repo) = temp_repo().unwrap();
    ///
    /// // Create some ADRs
    /// repo.new_adr("Use Rust").unwrap();
    /// repo.new_adr("Use PostgreSQL").unwrap();
    ///
    /// let adrs = repo.list().unwrap();
    /// assert_eq!(adrs.len(), 3); // 1 from init + 2 new
    /// assert_eq!(adrs[1].title, "Use Rust");
    /// assert_eq!(adrs[2].title, "Use PostgreSQL");
    /// ```
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

    /// Returns the next available ADR number.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use adrs_core::doctest_helpers::temp_repo;
    /// let (_temp, repo) = temp_repo().unwrap();
    ///
    /// // After init, next number is 2
    /// assert_eq!(repo.next_number().unwrap(), 2);
    ///
    /// repo.new_adr("Second ADR").unwrap();
    /// assert_eq!(repo.next_number().unwrap(), 3);
    /// ```
    pub fn next_number(&self) -> Result<u32> {
        let adrs = self.list()?;
        Ok(adrs.last().map(|a| a.number + 1).unwrap_or(1))
    }

    /// Finds an ADR by number.
    ///
    /// # Errors
    ///
    /// Returns [`Error::AdrNotFound`] if no ADR with the given number exists.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use adrs_core::doctest_helpers::temp_repo;
    /// let (_temp, repo) = temp_repo().unwrap();
    ///
    /// // ADR 1 is created by init
    /// let adr = repo.get(1).unwrap();
    /// assert_eq!(adr.number, 1);
    ///
    /// // Non-existent ADR returns error
    /// assert!(repo.get(999).is_err());
    /// ```
    pub fn get(&self, number: u32) -> Result<Adr> {
        let adrs = self.list()?;
        adrs.into_iter()
            .find(|a| a.number == number)
            .ok_or_else(|| Error::AdrNotFound(number.to_string()))
    }

    /// Finds an ADR by query (number or fuzzy title match).
    ///
    /// First tries to parse the query as a number. If that fails,
    /// performs fuzzy matching on ADR titles.
    ///
    /// # Errors
    ///
    /// - [`Error::AdrNotFound`] if no matching ADR is found
    /// - [`Error::AmbiguousAdr`] if multiple ADRs match equally well
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use adrs_core::doctest_helpers::temp_repo;
    /// let (_temp, repo) = temp_repo().unwrap();
    /// repo.new_adr("Use PostgreSQL for persistence").unwrap();
    ///
    /// // Find by number
    /// let adr = repo.find("2").unwrap();
    /// assert_eq!(adr.number, 2);
    ///
    /// // Find by fuzzy title match
    /// let adr = repo.find("postgres").unwrap();
    /// assert_eq!(adr.title, "Use PostgreSQL for persistence");
    /// ```
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

    /// Resolve link target titles and filenames for an ADR's links.
    fn resolve_link_titles(&self, adr: &Adr) -> HashMap<u32, (String, String)> {
        let mut map = HashMap::new();
        for link in &adr.links {
            if map.contains_key(&link.target) {
                continue;
            }
            if let Ok(target_adr) = self.get(link.target) {
                map.insert(
                    link.target,
                    (target_adr.title.clone(), target_adr.filename()),
                );
            }
        }
        map
    }

    /// Create a new ADR.
    pub fn create(&self, adr: &Adr) -> Result<PathBuf> {
        let path = self.adr_path().join(adr.filename());

        let link_titles = self.resolve_link_titles(adr);
        let content = self
            .template_engine
            .render(adr, &self.config, &link_titles)?;
        fs::write(&path, content)?;

        Ok(path)
    }

    /// Creates a new ADR with the given title.
    ///
    /// Automatically assigns the next available number and creates the file.
    ///
    /// # Returns
    ///
    /// A tuple of the created [`Adr`] and the path to the file.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use adrs_core::doctest_helpers::temp_repo;
    /// # use adrs_core::AdrStatus;
    /// let (_temp, repo) = temp_repo().unwrap();
    ///
    /// let (adr, path) = repo.new_adr("Use Rust for implementation").unwrap();
    ///
    /// assert_eq!(adr.number, 2); // 1 is from init
    /// assert_eq!(adr.status, AdrStatus::Proposed);
    /// assert!(path.exists());
    /// ```
    pub fn new_adr(&self, title: impl Into<String>) -> Result<(Adr, PathBuf)> {
        let number = self.next_number()?;
        let adr = Adr::new(number, title);
        let path = self.create(&adr)?;
        Ok((adr, path))
    }

    /// Creates a new ADR that supersedes another.
    ///
    /// The new ADR gets a "Supersedes" link to the old ADR, and the old
    /// ADR's status is changed to Superseded with a "Superseded by" link.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use adrs_core::doctest_helpers::temp_repo;
    /// # use adrs_core::{AdrStatus, LinkKind};
    /// let (_temp, repo) = temp_repo().unwrap();
    /// repo.new_adr("Use MySQL").unwrap();
    ///
    /// // Supersede ADR 2 with a new decision
    /// let (new_adr, _) = repo.supersede("Use PostgreSQL instead", 2).unwrap();
    ///
    /// // New ADR links to old
    /// assert_eq!(new_adr.links[0].target, 2);
    /// assert_eq!(new_adr.links[0].kind, LinkKind::Supersedes);
    ///
    /// // Old ADR is now superseded
    /// let old_adr = repo.get(2).unwrap();
    /// assert_eq!(old_adr.status, AdrStatus::Superseded);
    /// ```
    pub fn supersede(&self, title: impl Into<String>, superseded: u32) -> Result<(Adr, PathBuf)> {
        let number = self.next_number()?;
        let mut adr = Adr::new(number, title);
        adr.add_link(AdrLink::new(superseded, LinkKind::Supersedes));

        // Create the new ADR first so its file exists on disk when
        // the old ADR's "Superseded by" link is resolved.
        let path = self.create(&adr)?;

        // Now update the superseded ADR — the new ADR is on disk so
        // its title and filename can be resolved for the link.
        let mut old_adr = self.get(superseded)?;
        old_adr.status = AdrStatus::Superseded;
        old_adr.add_link(AdrLink::new(number, LinkKind::SupersededBy));
        self.update_metadata(&old_adr)?;

        Ok((adr, path))
    }

    /// Changes the status of an ADR.
    ///
    /// If the new status is `Superseded` and `superseded_by` is provided,
    /// a "Superseded by" link will be added automatically.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use adrs_core::doctest_helpers::temp_repo;
    /// # use adrs_core::AdrStatus;
    /// let (_temp, repo) = temp_repo().unwrap();
    ///
    /// // Change status from Accepted to Deprecated
    /// repo.set_status(1, AdrStatus::Deprecated, None).unwrap();
    ///
    /// let adr = repo.get(1).unwrap();
    /// assert_eq!(adr.status, AdrStatus::Deprecated);
    /// ```
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

        self.update_metadata(&adr)
    }

    /// Links two ADRs together with bidirectional references.
    ///
    /// Creates a link from `source` to `target` with `source_kind`,
    /// and a reverse link from `target` to `source` with `target_kind`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use adrs_core::doctest_helpers::temp_repo;
    /// # use adrs_core::LinkKind;
    /// let (_temp, repo) = temp_repo().unwrap();
    /// repo.new_adr("API Design").unwrap();
    /// repo.new_adr("API Implementation").unwrap();
    ///
    /// // ADR 3 amends ADR 2
    /// repo.link(3, 2, LinkKind::Amends, LinkKind::AmendedBy).unwrap();
    ///
    /// let adr3 = repo.get(3).unwrap();
    /// assert_eq!(adr3.links[0].target, 2);
    /// assert_eq!(adr3.links[0].kind, LinkKind::Amends);
    ///
    /// let adr2 = repo.get(2).unwrap();
    /// assert_eq!(adr2.links[0].target, 3);
    /// assert_eq!(adr2.links[0].kind, LinkKind::AmendedBy);
    /// ```
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

        self.update_metadata(&source_adr)?;
        self.update_metadata(&target_adr)?;

        Ok(())
    }

    /// Updates an existing ADR by re-rendering it completely.
    ///
    /// This rewrites the entire ADR file using the current template.
    /// For partial updates that preserve custom content, use
    /// [`update_metadata`](Self::update_metadata) instead.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use adrs_core::doctest_helpers::temp_repo;
    /// # use adrs_core::AdrStatus;
    /// let (_temp, repo) = temp_repo().unwrap();
    ///
    /// let mut adr = repo.get(1).unwrap();
    /// adr.context = "Updated context explaining the decision.".into();
    /// adr.status = AdrStatus::Deprecated;
    ///
    /// let path = repo.update(&adr).unwrap();
    /// let content = std::fs::read_to_string(path).unwrap();
    /// assert!(content.contains("Updated context"));
    /// assert!(content.contains("Deprecated"));
    /// ```
    pub fn update(&self, adr: &Adr) -> Result<PathBuf> {
        let path = adr
            .path
            .clone()
            .unwrap_or_else(|| self.adr_path().join(adr.filename()));

        let link_titles = self.resolve_link_titles(adr);
        let content = self
            .template_engine
            .render(adr, &self.config, &link_titles)?;
        fs::write(&path, content)?;

        Ok(path)
    }

    /// Reads the raw content of an ADR file.
    ///
    /// Returns the complete file content as a string. Use this when you
    /// need access to the raw markdown, including custom sections not
    /// captured in the [`Adr`] struct.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use adrs_core::doctest_helpers::temp_repo;
    /// let (_temp, repo) = temp_repo().unwrap();
    ///
    /// let adr = repo.get(1).unwrap();
    /// let content = repo.read_content(&adr).unwrap();
    ///
    /// assert!(content.contains("Record architecture decisions"));
    /// assert!(content.contains("## Status"));
    /// ```
    pub fn read_content(&self, adr: &Adr) -> Result<String> {
        let path = adr
            .path
            .as_ref()
            .cloned()
            .unwrap_or_else(|| self.adr_path().join(adr.filename()));

        Ok(fs::read_to_string(path)?)
    }

    /// Writes raw content to an ADR file.
    ///
    /// Overwrites the file with the given content. Use this for direct
    /// file manipulation when the standard update methods don't suffice.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use adrs_core::doctest_helpers::temp_repo;
    /// let (_temp, repo) = temp_repo().unwrap();
    ///
    /// let adr = repo.get(1).unwrap();
    /// let new_content = "# 1. Modified Title\n\n## Status\n\nAccepted\n";
    ///
    /// repo.write_content(&adr, new_content).unwrap();
    ///
    /// let content = repo.read_content(&adr).unwrap();
    /// assert!(content.contains("Modified Title"));
    /// ```
    pub fn write_content(&self, adr: &Adr, content: &str) -> Result<PathBuf> {
        let path = adr
            .path
            .as_ref()
            .cloned()
            .unwrap_or_else(|| self.adr_path().join(adr.filename()));

        fs::write(&path, content)?;
        Ok(path)
    }

    /// Updates only the metadata (status, links, tags) of an ADR file.
    ///
    /// Unlike [`update`](Self::update), this method preserves all other
    /// content byte-for-byte, including custom sections, formatting,
    /// and comments.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use adrs_core::doctest_helpers::temp_repo;
    /// # use adrs_core::AdrStatus;
    /// let (_temp, repo) = temp_repo().unwrap();
    ///
    /// let mut adr = repo.get(1).unwrap();
    /// adr.status = AdrStatus::Deprecated;
    /// adr.set_tags(vec!["legacy".into()]);
    ///
    /// repo.update_metadata(&adr).unwrap();
    ///
    /// let updated = repo.get(1).unwrap();
    /// assert_eq!(updated.status, AdrStatus::Deprecated);
    /// ```
    pub fn update_metadata(&self, adr: &Adr) -> Result<PathBuf> {
        let path = adr
            .path
            .clone()
            .unwrap_or_else(|| self.adr_path().join(adr.filename()));

        let content = fs::read_to_string(&path)?;

        let updated = if content.starts_with("---\n") {
            self.update_frontmatter_metadata(adr, &content)?
        } else {
            self.update_legacy_metadata(adr, &content)?
        };

        fs::write(&path, updated)?;
        Ok(path)
    }

    /// Surgically update metadata fields in a YAML frontmatter file.
    ///
    /// Replaces only `status:`, `links:`, and `tags:` blocks in the frontmatter.
    /// YAML comments (e.g., SPDX headers), unknown fields, and the entire
    /// markdown body are preserved untouched.
    fn update_frontmatter_metadata(&self, adr: &Adr, content: &str) -> Result<String> {
        // Split into frontmatter and body at the closing `---`
        let Some(rest) = content.strip_prefix("---\n") else {
            return Err(Error::InvalidFormat {
                path: Default::default(),
                reason: "Missing opening frontmatter delimiter".into(),
            });
        };

        let Some(end_idx) = rest.find("\n---\n").or_else(|| {
            // Handle case where closing delimiter is at end of file with no trailing newline
            if rest.ends_with("\n---") {
                Some(rest.len() - 3)
            } else {
                None
            }
        }) else {
            return Err(Error::InvalidFormat {
                path: Default::default(),
                reason: "Missing closing frontmatter delimiter".into(),
            });
        };

        let yaml_block = &rest[..end_idx + 1]; // include trailing \n
        let after_yaml = &rest[end_idx..]; // starts with \n---\n...

        // 1. Replace status line
        let new_status = format!("status: {}", adr.status.to_string().to_lowercase());
        let yaml_block = FM_STATUS_RE.replace(yaml_block, new_status.as_str());

        // 2. Replace or remove links block
        let links_yaml = Self::format_links_yaml(&adr.links);
        let yaml_block = if FM_LINKS_RE.is_match(&yaml_block) {
            FM_LINKS_RE
                .replace(&yaml_block, links_yaml.as_str())
                .into_owned()
        } else if !links_yaml.is_empty() {
            // Append links before end of frontmatter
            let mut s = yaml_block.into_owned();
            if !s.ends_with('\n') {
                s.push('\n');
            }
            s.push_str(&links_yaml);
            s
        } else {
            yaml_block.into_owned()
        };

        // 3. Replace or remove tags block
        let tags_yaml = Self::format_tags_yaml(&adr.tags);
        let yaml_block = if FM_TAGS_RE.is_match(&yaml_block) {
            FM_TAGS_RE
                .replace(&yaml_block, tags_yaml.as_str())
                .into_owned()
        } else if !tags_yaml.is_empty() {
            let mut s = yaml_block;
            if !s.ends_with('\n') {
                s.push('\n');
            }
            s.push_str(&tags_yaml);
            s
        } else {
            yaml_block
        };

        let yaml_block = yaml_block.trim_end_matches('\n');
        Ok(format!("---\n{}{}", yaml_block, after_yaml))
    }

    /// Surgically update metadata in a legacy (no-frontmatter) ADR file.
    ///
    /// Replaces the content between `## Status` and the next `## ` heading
    /// with the new status and link lines. All other sections pass through untouched.
    fn update_legacy_metadata(&self, adr: &Adr, content: &str) -> Result<String> {
        let lines: Vec<&str> = content.lines().collect();
        let mut result = String::with_capacity(content.len());

        // Find the ## Status section
        let status_idx = lines.iter().position(|l| {
            l.trim().eq_ignore_ascii_case("## Status") || l.trim().eq_ignore_ascii_case("## STATUS")
        });

        let Some(status_idx) = status_idx else {
            // No status section found -- just return content unchanged
            return Ok(content.to_string());
        };

        // Find the next ## heading after status
        let next_heading_idx = lines[status_idx + 1..]
            .iter()
            .position(|l| l.starts_with("## "))
            .map(|i| i + status_idx + 1);

        // Write everything before the status section (including the ## Status line)
        for line in &lines[..=status_idx] {
            result.push_str(line);
            result.push('\n');
        }

        // Write new status content
        result.push('\n');
        result.push_str(&adr.status.to_string());
        result.push('\n');

        // Write link lines with resolved titles
        let link_titles = self.resolve_link_titles(adr);
        for link in &adr.links {
            result.push('\n');
            if let Some((title, filename)) = link_titles.get(&link.target) {
                result.push_str(&format!(
                    "{} [{}. {}]({})",
                    link.kind, link.target, title, filename
                ));
            } else {
                result.push_str(&format!(
                    "{} [{}. ...]({:04}-....md)",
                    link.kind, link.target, link.target
                ));
            }
            result.push('\n');
        }

        // Write everything from the next heading onward
        if let Some(next_idx) = next_heading_idx {
            result.push('\n');
            for (i, line) in lines[next_idx..].iter().enumerate() {
                result.push_str(line);
                // Preserve trailing newline behavior
                if next_idx + i < lines.len() - 1 || content.ends_with('\n') {
                    result.push('\n');
                }
            }
        } else if content.ends_with('\n') {
            // No next heading, but original ended with newline
        }

        Ok(result)
    }

    /// Format links as YAML block for frontmatter insertion.
    fn format_links_yaml(links: &[AdrLink]) -> String {
        if links.is_empty() {
            return String::new();
        }
        let mut s = String::from("links:\n");
        for link in links {
            let kind_str = match &link.kind {
                LinkKind::Supersedes => "supersedes",
                LinkKind::SupersededBy => "supersededby",
                LinkKind::Amends => "amends",
                LinkKind::AmendedBy => "amendedby",
                LinkKind::RelatesTo => "relatesto",
                LinkKind::Custom(c) => c.as_str(),
            };
            s.push_str(&format!(
                "  - target: {}\n    kind: {}\n",
                link.target, kind_str
            ));
        }
        s
    }

    /// Format tags as YAML block for frontmatter insertion.
    fn format_tags_yaml(tags: &[String]) -> String {
        if tags.is_empty() {
            return String::new();
        }
        let mut s = String::from("tags:\n");
        for tag in tags {
            s.push_str(&format!("  - {}\n", tag));
        }
        s
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

    // ========== Link Resolution Tests (Issue #180) ==========

    #[test]
    fn test_supersede_generates_functional_links() {
        let temp = TempDir::new().unwrap();
        let repo = Repository::init(temp.path(), None, false).unwrap();

        // Create ADR 2, then supersede it with ADR 3
        repo.new_adr("Use MySQL for persistence").unwrap();
        repo.supersede("Use PostgreSQL instead", 2).unwrap();

        // Check the new ADR (3) has a functional "Supersedes" link to ADR 2
        let new_content =
            fs::read_to_string(repo.adr_path().join("0003-use-postgresql-instead.md")).unwrap();
        assert!(
            new_content.contains(
                "Supersedes [2. Use MySQL for persistence](0002-use-mysql-for-persistence.md)"
            ),
            "New ADR should have functional Supersedes link. Got:\n{new_content}"
        );

        // Check the old ADR (2) has a functional "Superseded by" link to ADR 3
        let old_content =
            fs::read_to_string(repo.adr_path().join("0002-use-mysql-for-persistence.md")).unwrap();
        assert!(
            old_content.contains(
                "Superseded by [3. Use PostgreSQL instead](0003-use-postgresql-instead.md)"
            ),
            "Old ADR should have functional Superseded by link. Got:\n{old_content}"
        );
    }

    #[test]
    fn test_link_generates_functional_links() {
        let temp = TempDir::new().unwrap();
        let repo = Repository::init(temp.path(), None, false).unwrap();

        repo.new_adr("Use REST API").unwrap();
        repo.new_adr("Use JSON for API responses").unwrap();

        repo.link(3, 2, LinkKind::Amends, LinkKind::AmendedBy)
            .unwrap();

        // Check source ADR has functional link
        let source_content =
            fs::read_to_string(repo.adr_path().join("0003-use-json-for-api-responses.md")).unwrap();
        assert!(
            source_content.contains("Amends [2. Use REST API](0002-use-rest-api.md)"),
            "Source ADR should have functional Amends link. Got:\n{source_content}"
        );

        // Check target ADR has functional reverse link
        let target_content =
            fs::read_to_string(repo.adr_path().join("0002-use-rest-api.md")).unwrap();
        assert!(
            target_content.contains(
                "Amended by [3. Use JSON for API responses](0003-use-json-for-api-responses.md)"
            ),
            "Target ADR should have functional Amended by link. Got:\n{target_content}"
        );
    }

    #[test]
    fn test_set_status_superseded_generates_functional_link() {
        let temp = TempDir::new().unwrap();
        let repo = Repository::init(temp.path(), None, false).unwrap();

        repo.new_adr("First Decision").unwrap();
        repo.new_adr("Second Decision").unwrap();

        repo.set_status(2, AdrStatus::Superseded, Some(3)).unwrap();

        let content = fs::read_to_string(repo.adr_path().join("0002-first-decision.md")).unwrap();
        assert!(
            content.contains("Superseded by [3. Second Decision](0003-second-decision.md)"),
            "ADR should have functional Superseded by link. Got:\n{content}"
        );
    }

    #[test]
    fn test_supersede_chain_generates_functional_links() {
        let temp = TempDir::new().unwrap();
        let repo = Repository::init(temp.path(), None, false).unwrap();

        // ADR 1 is "Record architecture decisions" (from init)
        // Create ADR 2
        repo.new_adr("Use SQLite").unwrap();
        // ADR 3 supersedes ADR 2
        repo.supersede("Use PostgreSQL", 2).unwrap();
        // ADR 4 supersedes ADR 3
        repo.supersede("Use CockroachDB", 3).unwrap();

        // Check ADR 3 has both directions
        let adr3_content =
            fs::read_to_string(repo.adr_path().join("0003-use-postgresql.md")).unwrap();
        assert!(
            adr3_content.contains("Supersedes [2. Use SQLite](0002-use-sqlite.md)"),
            "ADR 3 should supersede ADR 2. Got:\n{adr3_content}"
        );
        assert!(
            adr3_content.contains("Superseded by [4. Use CockroachDB](0004-use-cockroachdb.md)"),
            "ADR 3 should be superseded by ADR 4. Got:\n{adr3_content}"
        );
    }

    #[test]
    fn test_ng_mode_supersede_generates_functional_links() {
        let temp = TempDir::new().unwrap();
        let repo = Repository::init(temp.path(), None, true).unwrap();

        repo.new_adr("Use MySQL").unwrap();
        repo.supersede("Use PostgreSQL", 2).unwrap();

        // Check the new ADR has functional links in both frontmatter and body
        let new_content =
            fs::read_to_string(repo.adr_path().join("0003-use-postgresql.md")).unwrap();

        // Body should have functional markdown link
        assert!(
            new_content.contains("Supersedes [2. Use MySQL](0002-use-mysql.md)"),
            "NG mode should have functional link in body. Got:\n{new_content}"
        );
        // Frontmatter should have structured link
        assert!(new_content.contains("links:"));
        assert!(new_content.contains("target: 2"));
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

    // ========== Metadata Preservation Tests (issue #187) ==========

    #[test]
    fn test_set_status_preserves_madr_body() {
        let temp = TempDir::new().unwrap();
        let repo = Repository::init(temp.path(), None, true).unwrap();

        let madr_content = r#"---
number: 2
title: Use Redis for caching
date: 2026-01-15
status: proposed
---

# Use Redis for caching

## Context and Problem Statement

We need a **fast** caching layer for our [API](https://api.example.com).

## Considered Options

* Redis
* Memcached
* In-memory cache

## Decision Outcome

Chosen option: "Redis", because it supports data structures beyond simple key-value.

### Consequences

* Good, because it provides pub/sub
* Bad, because it adds operational complexity

## Pros and Cons of the Options

### Redis

* Good, because it supports complex data types
* Bad, because it requires a separate server

### Memcached

* Good, because it's simpler
* Bad, because it only supports strings
"#;
        let adr_path = repo.adr_path().join("0002-use-redis-for-caching.md");
        fs::write(&adr_path, madr_content).unwrap();

        // Change status
        repo.set_status(2, AdrStatus::Accepted, None).unwrap();

        let result = fs::read_to_string(&adr_path).unwrap();

        // Status should be updated
        assert!(result.contains("status: accepted"));
        assert!(!result.contains("status: proposed"));

        // Body should be completely preserved
        let body_start = result.find("\n# Use Redis").unwrap();
        let original_body_start = madr_content.find("\n# Use Redis").unwrap();
        assert_eq!(
            &result[body_start..],
            &madr_content[original_body_start..],
            "Body content was modified"
        );
    }

    #[test]
    fn test_set_status_preserves_yaml_comments() {
        let temp = TempDir::new().unwrap();
        let repo = Repository::init(temp.path(), None, true).unwrap();

        let content_with_comments = r#"---
# SPDX-License-Identifier: MIT
# SPDX-FileCopyrightText: 2026 Example Corp
number: 2
title: Use MADR format
date: 2026-01-15
status: proposed
---

## Context and Problem Statement

We need a standard ADR format.

## Decision Outcome

Use MADR 4.0.0.
"#;
        let adr_path = repo.adr_path().join("0002-use-madr-format.md");
        fs::write(&adr_path, content_with_comments).unwrap();

        repo.set_status(2, AdrStatus::Accepted, None).unwrap();

        let result = fs::read_to_string(&adr_path).unwrap();

        // YAML comments must be preserved
        assert!(
            result.contains("# SPDX-License-Identifier: MIT"),
            "SPDX comment was destroyed"
        );
        assert!(
            result.contains("# SPDX-FileCopyrightText: 2026 Example Corp"),
            "Copyright comment was destroyed"
        );
        assert!(result.contains("status: accepted"));
    }

    #[test]
    fn test_set_status_preserves_markdown_links() {
        let temp = TempDir::new().unwrap();
        let repo = Repository::init(temp.path(), None, true).unwrap();

        let content = r#"---
number: 2
title: Use PostgreSQL
date: 2026-01-15
status: proposed
---

## Context

See the [PostgreSQL docs](https://www.postgresql.org/docs/) for details.

Also see [RFC 7159](https://tools.ietf.org/html/rfc7159) and `inline code`.

## Decision

We will use **PostgreSQL** version `16.x`.

## Consequences

- [Monitoring guide](https://example.com/monitoring)
- Performance benchmarks in [this report](./benchmarks.md)
"#;
        let adr_path = repo.adr_path().join("0002-use-postgresql.md");
        fs::write(&adr_path, content).unwrap();

        repo.set_status(2, AdrStatus::Accepted, None).unwrap();

        let result = fs::read_to_string(&adr_path).unwrap();

        assert!(result.contains("[PostgreSQL docs](https://www.postgresql.org/docs/)"));
        assert!(result.contains("[RFC 7159](https://tools.ietf.org/html/rfc7159)"));
        assert!(result.contains("`inline code`"));
        assert!(result.contains("**PostgreSQL**"));
        assert!(result.contains("[Monitoring guide](https://example.com/monitoring)"));
        assert!(result.contains("[this report](./benchmarks.md)"));
    }

    #[test]
    fn test_link_preserves_body_content() {
        let temp = TempDir::new().unwrap();
        let repo = Repository::init(temp.path(), None, true).unwrap();

        let content_1 = r#"---
number: 2
title: First decision
date: 2026-01-15
status: accepted
---

## Context

Custom context with **bold** and [links](https://example.com).

## Decision

A detailed decision paragraph.

## Consequences

- Important consequence 1
- Important consequence 2
"#;
        let content_2 = r#"---
number: 3
title: Second decision
date: 2026-01-16
status: accepted
---

## Context

Different context entirely.

## Decision

Another decision.

## Consequences

None significant.
"#;
        fs::write(repo.adr_path().join("0002-first-decision.md"), content_1).unwrap();
        fs::write(repo.adr_path().join("0003-second-decision.md"), content_2).unwrap();

        repo.link(2, 3, LinkKind::Amends, LinkKind::AmendedBy)
            .unwrap();

        let result_1 = fs::read_to_string(repo.adr_path().join("0002-first-decision.md")).unwrap();
        let result_2 = fs::read_to_string(repo.adr_path().join("0003-second-decision.md")).unwrap();

        // Bodies must be intact
        assert!(result_1.contains("Custom context with **bold** and [links](https://example.com)"));
        assert!(result_1.contains("A detailed decision paragraph."));
        assert!(result_2.contains("Different context entirely."));
        assert!(result_2.contains("None significant."));

        // Links must be present in frontmatter
        assert!(result_1.contains("links:"));
        assert!(result_1.contains("target: 3"));
        assert!(result_2.contains("links:"));
        assert!(result_2.contains("target: 2"));
    }

    #[test]
    fn test_supersede_preserves_old_adr_body() {
        let temp = TempDir::new().unwrap();
        let repo = Repository::init(temp.path(), None, true).unwrap();

        let rich_content = r#"---
number: 2
title: Original approach
date: 2026-01-15
status: accepted
---

## Context and Problem Statement

This has **rich** markdown with [links](https://example.com).

```rust
fn important_code() -> bool {
    true
}
```

## Decision Outcome

We chose the original approach.

| Criteria | Score |
|----------|-------|
| Speed    | 9/10  |
| Safety   | 8/10  |
"#;
        fs::write(
            repo.adr_path().join("0002-original-approach.md"),
            rich_content,
        )
        .unwrap();

        repo.supersede("Better approach", 2).unwrap();

        let old_content =
            fs::read_to_string(repo.adr_path().join("0002-original-approach.md")).unwrap();

        // Old ADR body must be preserved
        assert!(old_content.contains("```rust"));
        assert!(old_content.contains("fn important_code()"));
        assert!(old_content.contains("| Criteria | Score |"));
        assert!(old_content.contains("[links](https://example.com)"));

        // Status and links must be updated
        assert!(old_content.contains("status: superseded"));
        assert!(old_content.contains("target: 3"));
    }

    #[test]
    fn test_set_status_legacy_preserves_sections() {
        let temp = TempDir::new().unwrap();
        let repo = Repository::init(temp.path(), None, false).unwrap();

        let legacy_content = r#"# 2. Use Rust for backend

Date: 2026-01-15

## Status

Proposed

## Context

We need a fast, safe language for our backend services.

See the [Rust book](https://doc.rust-lang.org/book/) for details.

## Decision

We will use **Rust** with the `tokio` runtime.

```toml
[dependencies]
tokio = { version = "1", features = ["full"] }
```

## Consequences

- Type safety prevents many bugs at compile time
- Learning curve for team members
"#;
        let adr_path = repo.adr_path().join("0002-use-rust-for-backend.md");
        fs::write(&adr_path, legacy_content).unwrap();

        repo.set_status(2, AdrStatus::Accepted, None).unwrap();

        let result = fs::read_to_string(&adr_path).unwrap();

        // Status should change
        assert!(result.contains("Accepted"));

        // Other sections must be preserved exactly
        assert!(result.contains("[Rust book](https://doc.rust-lang.org/book/)"));
        assert!(result.contains("**Rust**"));
        assert!(result.contains("`tokio`"));
        assert!(result.contains("```toml"));
        assert!(result.contains("tokio = { version = \"1\", features = [\"full\"] }"));
        assert!(result.contains("Type safety prevents many bugs"));
    }

    #[test]
    fn test_set_status_frontmatter_with_existing_links() {
        let temp = TempDir::new().unwrap();
        let repo = Repository::init(temp.path(), None, true).unwrap();

        let content = r#"---
number: 2
title: Updated approach
date: 2026-01-15
status: proposed
links:
  - target: 1
    kind: amends
---

## Context

Context.

## Decision

Decision.
"#;
        let adr_path = repo.adr_path().join("0002-updated-approach.md");
        fs::write(&adr_path, content).unwrap();

        // Just change status, links should be preserved
        repo.set_status(2, AdrStatus::Accepted, None).unwrap();

        let result = fs::read_to_string(&adr_path).unwrap();
        assert!(result.contains("status: accepted"));
        assert!(result.contains("links:"));
        assert!(result.contains("target: 1"));
        assert!(result.contains("kind: amends"));
        // No extra blank line before closing ---
        assert!(
            !result.contains("\n\n---"),
            "Should not have extra blank line before closing ---: {:?}",
            result
        );
    }

    #[test]
    fn test_set_status_no_extra_newline_before_separator() {
        let temp = TempDir::new().unwrap();
        let repo = Repository::init(temp.path(), None, true).unwrap();

        let content = "---\nnumber: 2\ntitle: Test\ndate: 2026-01-15\nstatus: proposed\n---\n\n## Context\n\nContext.\n";
        let adr_path = repo.adr_path().join("0002-test.md");
        fs::write(&adr_path, content).unwrap();

        repo.set_status(2, AdrStatus::Accepted, None).unwrap();

        let result = fs::read_to_string(&adr_path).unwrap();
        assert!(result.contains("status: accepted"));
        // Frontmatter should close cleanly without extra blank line (#192)
        assert!(
            result.contains("\n---\n"),
            "Should have clean closing separator: {:?}",
            result
        );
        assert!(
            !result.contains("\n\n---"),
            "Should not have extra blank line before closing ---: {:?}",
            result
        );
    }

    #[test]
    fn test_update_metadata_adds_tags_to_frontmatter() {
        let temp = TempDir::new().unwrap();
        let repo = Repository::init(temp.path(), None, true).unwrap();

        let content = r#"---
number: 2
title: Tagged ADR
date: 2026-01-15
status: proposed
---

## Context

Context.
"#;
        let adr_path = repo.adr_path().join("0002-tagged-adr.md");
        fs::write(&adr_path, content).unwrap();

        let mut adr = repo.get(2).unwrap();
        adr.set_tags(vec!["security".into(), "api".into()]);
        repo.update_metadata(&adr).unwrap();

        let result = fs::read_to_string(&adr_path).unwrap();
        assert!(result.contains("tags:"));
        assert!(result.contains("  - security"));
        assert!(result.contains("  - api"));
        // Body preserved
        assert!(result.contains("## Context\n\nContext."));
    }
}
