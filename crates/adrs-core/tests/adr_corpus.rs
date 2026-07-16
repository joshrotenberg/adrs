//! Corpus regression tests over `tests/fixtures/adr-corpus/`.
//!
//! The corpus is a set of realistic ADR files (Nygard and MADR 4.0 frontmatter,
//! various statuses and link styles) exercised as a whole through the parser
//! and the repository read/write paths. See issue #318 for why these fixtures
//! exist and how they were wired up.
//!
//! Pinning policy: some assertions record behavior that is known to be lossy
//! or surprising rather than desirable. Those are marked KNOWN-LOSSY or
//! KNOWN-QUIRK with the tracking issue. When one of those issues is fixed the
//! corresponding assertion here is expected to fail and should be tightened
//! deliberately, not worked around.

use adrs_core::{AdrStatus, BodySectionPatch, LinkKind, Parser, Repository};
use std::fs;
use std::path::{Path, PathBuf};

fn corpus_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/adr-corpus")
}

/// All numbered fixture files (`NNNN-*.md`), sorted. Skips the README.
fn corpus_files() -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = fs::read_dir(corpus_dir())
        .expect("corpus dir readable")
        .map(|e| e.expect("dir entry").path())
        .filter(|p| {
            p.extension().is_some_and(|e| e == "md")
                && p.file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| n.chars().take(4).all(|c| c.is_ascii_digit()))
        })
        .collect();
    files.sort();
    files
}

fn fixture_path(number: u32) -> PathBuf {
    corpus_files()
        .into_iter()
        .find(|f| {
            f.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with(&format!("{number:04}")))
        })
        .unwrap_or_else(|| panic!("no corpus fixture numbered {number:04}"))
}

fn parse_fixture(number: u32) -> adrs_core::Adr {
    let path = fixture_path(number);
    Parser::new()
        .parse_file(&path)
        .unwrap_or_else(|e| panic!("{} failed to parse: {e}", path.display()))
}

/// Copy the whole corpus into a temp directory laid out as an ADR repository.
fn corpus_repo() -> (tempfile::TempDir, Repository) {
    let tmp = tempfile::tempdir().expect("tempdir");
    let adr_dir = tmp.path().join("doc/adr");
    fs::create_dir_all(&adr_dir).expect("create doc/adr");
    for f in corpus_files() {
        fs::copy(&f, adr_dir.join(f.file_name().unwrap())).expect("copy fixture");
    }
    let repo = Repository::open_or_default(tmp.path());
    (tmp, repo)
}

#[test]
fn corpus_is_complete() {
    let files = corpus_files();
    assert_eq!(
        files.len(),
        16,
        "expected 16 corpus fixtures, add new ones to the README table"
    );
    for (i, f) in files.iter().enumerate() {
        let name = f.file_name().unwrap().to_str().unwrap();
        assert!(
            name.starts_with(&format!("{:04}-", i + 1)),
            "corpus numbering gap: expected {:04}-*, found {name}",
            i + 1
        );
    }
    assert!(corpus_dir().join("README.md").exists());
}

#[test]
fn every_corpus_file_parses() {
    for f in corpus_files() {
        Parser::new()
            .parse_file(&f)
            .unwrap_or_else(|e| panic!("{} failed to parse: {e}", f.display()));
    }
}

#[test]
fn nygard_fixtures_parse_as_expected() {
    let adr = parse_fixture(1);
    assert_eq!(adr.number, 1);
    assert_eq!(adr.title, "Record architecture decisions");
    assert_eq!(adr.status, AdrStatus::Accepted);
    assert!(adr.links.is_empty());
    assert!(adr.context.contains("record the architectural decisions"));

    let adr = parse_fixture(2);
    assert_eq!(adr.status, AdrStatus::Accepted);
    assert_eq!(adr.links.len(), 1);
    assert_eq!(adr.links[0].target, 1);
    assert_eq!(adr.links[0].kind, LinkKind::Supersedes);
    // Legacy list formatting survives in section text.
    assert!(
        adr.consequences
            .contains("- Team needs to ensure PostgreSQL expertise")
    );

    let adr = parse_fixture(3);
    assert_eq!(adr.status, AdrStatus::Proposed);
    assert!(
        adr.context
            .contains("- High performance for data processing")
    );

    let adr = parse_fixture(5);
    assert_eq!(adr.links.len(), 1);
    assert_eq!(adr.links[0].target, 3);
    assert_eq!(adr.links[0].kind, LinkKind::Amends);

    // KNOWN-QUIRK: 0006 declares `Deprecated` but also carries a
    // `Superseded by [...]` line; the status parser lets the later
    // superseded marker win, so the explicit Deprecated is not preserved.
    let adr = parse_fixture(6);
    assert_eq!(adr.status, AdrStatus::Superseded);
    assert_eq!(adr.links.len(), 1);
    assert_eq!(adr.links[0].target, 5);
    assert_eq!(adr.links[0].kind, LinkKind::SupersededBy);
}

#[test]
fn frontmatter_fixtures_parse_as_expected() {
    let adr = parse_fixture(4);
    assert_eq!(adr.number, 4);
    assert_eq!(adr.title, "Use MADR format for ADRs");
    assert_eq!(adr.status, AdrStatus::Accepted);
    assert_eq!(adr.date.to_string(), "2024-02-15");
    assert_eq!(adr.decision_makers, ["Alice Smith", "Bob Jones"]);
    assert_eq!(adr.consulted, ["Carol White"]);
    assert_eq!(adr.informed, ["David Brown", "Eve Green"]);

    let adr = parse_fixture(7);
    assert_eq!(adr.date.to_string(), "2024-04-01");
    assert_eq!(adr.decision_makers, ["Security Team", "Platform Team"]);
    assert_eq!(adr.links.len(), 2);
    assert_eq!(adr.links[0].target, 2);
    assert_eq!(
        adr.links[0].description.as_deref(),
        Some("Database choice affects token storage")
    );
    // KNOWN-QUIRK (#323): kebab-case `kind: relates-to` deserializes as a
    // Custom kind instead of LinkKind::RelatesTo.
    assert_eq!(adr.links[0].kind, LinkKind::Custom("relates-to".into()));
}

#[test]
fn legacy_date_lines_are_parsed() {
    // The Nygard `Date: 2024-01-15` line under the H1 is now parsed into
    // adr.date instead of defaulting to today.
    let adr = parse_fixture(1);
    assert_eq!(adr.date.to_string(), "2024-01-15");
}

#[test]
fn fenced_heading_lookalikes_are_not_section_boundaries_on_read() {
    // 0008 embeds a fenced markdown example containing `## Context` and
    // `## Consequences` lines. The parser must keep fence content inside the
    // section that contains the fence.
    let adr = parse_fixture(8);
    assert!(
        adr.context.contains("## Consequences"),
        "fence content must stay in context"
    );
    assert!(adr.context.contains("Example consequences inside a fence."));
    assert!(
        adr.context.contains("Text after the fence still belongs"),
        "content after the fence must remain in the same section"
    );
    assert!(
        adr.consequences
            .contains("Tools must treat fence content as opaque text")
    );
}

#[test]
fn file_without_trailing_newline_parses() {
    let raw = fs::read(fixture_path(9)).unwrap();
    assert_ne!(
        raw.last(),
        Some(&b'\n'),
        "fixture must not end with a newline"
    );
    let adr = parse_fixture(9);
    assert_eq!(adr.number, 9);
    assert_eq!(adr.status, AdrStatus::Accepted);
    assert!(adr.consequences.contains("must not corrupt such files"));
}

#[test]
fn non_canonical_status_prose_pins_current_behavior() {
    // KNOWN-LOSSY (#310): unrecognized status prose is dropped on read and
    // the status defaults to Proposed. PR #311's rework should improve this;
    // tighten when it does.
    let adr = parse_fixture(10);
    assert_eq!(adr.status, AdrStatus::Proposed);
}

#[test]
fn crlf_frontmatter_parses_like_lf() {
    // A file with CRLF line endings parses its YAML frontmatter identically
    // to the same content with LF line endings. Constructed in memory so the
    // assertion is deterministic on every platform regardless of git eol
    // settings (a .gitattributes rule keeps the on-disk fixtures LF).
    let lf = fs::read_to_string(fixture_path(4)).unwrap();
    let crlf = lf.replace('\n', "\r\n");
    let adr = Parser::new().parse(&crlf).unwrap();
    assert_eq!(adr.status, AdrStatus::Accepted);
    assert_eq!(adr.decision_makers, ["Alice Smith", "Bob Jones"]);
}

#[test]
fn repository_lists_full_corpus() {
    let (_tmp, repo) = corpus_repo();
    let (adrs, errors) = repo.list_with_errors().unwrap();
    assert_eq!(adrs.len(), 16);
    assert!(errors.is_empty(), "corpus files must all load: {errors:?}");
    let numbers: Vec<u32> = adrs.iter().map(|a| a.number).collect();
    assert_eq!(numbers, (1..=16).collect::<Vec<u32>>());
}

#[test]
fn noop_metadata_update_is_byte_identical_for_well_formed_files() {
    // A get() followed by update_metadata() with nothing changed must leave
    // these files untouched, byte for byte. This is the core write-path
    // guarantee the corpus protects: canonical Nygard files, frontmatter
    // files with people fields (0004, 0011-0014), a legacy Amends link to a
    // hand-named file (0005), fence content (0008, 0015-0016), a file without
    // a trailing newline (0009), and frontmatter links with descriptions (0007).
    //
    let (tmp, repo) = corpus_repo();
    for number in [1u32, 2, 3, 4, 5, 7, 8, 9, 11, 12, 13, 14, 15, 16] {
        let file = tmp
            .path()
            .join("doc/adr")
            .join(fixture_path(number).file_name().unwrap());
        let before = fs::read(&file).unwrap();
        let adr = repo.get(number).unwrap();
        repo.update_metadata(&adr).unwrap();
        let after = fs::read(&file).unwrap();
        assert_eq!(
            before, after,
            "no-op update_metadata changed {:04} on disk",
            number
        );
    }
}

#[test]
fn noop_metadata_update_pins_known_rewrites() {
    // These fixtures are NOT byte-stable under a no-op update_metadata today.
    // Each case pins the current lossy behavior with its tracking issue.
    let (tmp, repo) = corpus_repo();
    let file = |n: u32| {
        tmp.path()
            .join("doc/adr")
            .join(fixture_path(n).file_name().unwrap())
    };

    // KNOWN-LOSSY: 0006's explicit `Deprecated` status is materialized as
    // `Superseded` (the parse-side quirk written back to disk). The H1 and
    // body prose still mention "Deprecated", so assert on the Status section.
    let adr = repo.get(6).unwrap();
    repo.update_metadata(&adr).unwrap();
    let after = fs::read_to_string(file(6)).unwrap();
    assert!(after.contains("## Status\n\nSuperseded"));
    assert!(
        !after.contains("## Status\n\nDeprecated"),
        "0006 keeps its Deprecated status now; tighten this pin"
    );

    // KNOWN-LOSSY (#310): non-canonical status prose is rewritten to the
    // parsed default.
    let adr = repo.get(10).unwrap();
    repo.update_metadata(&adr).unwrap();
    let after = fs::read_to_string(file(10)).unwrap();
    assert!(after.contains("Proposed"));
    assert!(
        !after.contains("Approved by the architecture board"),
        "status prose survives update_metadata now; tighten this pin"
    );
}

#[test]
fn status_change_preserves_people_yaml_forms() {
    // Josh's adrs status regression (review 4707905821 #1):
    // success exit must not orphan people-field YAML or drop the ADR from list.
    let (tmp, repo) = corpus_repo();
    let adr_dir = tmp.path().join("doc/adr");

    for number in [11u32, 12, 13, 14] {
        let file = adr_dir.join(fixture_path(number).file_name().unwrap());

        repo.set_status(number, AdrStatus::Accepted, None)
            .unwrap_or_else(|e| panic!("set_status({number}) failed: {e}"));

        let after = fs::read_to_string(&file).unwrap();
        assert!(
            after.contains("status: accepted"),
            "{number:04}: status should update"
        );

        // File must still parse and remain listable.
        let listed = repo.get(number).unwrap_or_else(|e| {
            panic!("{number:04}: ADR disappeared from list after set_status: {e}\n{after}")
        });
        assert_eq!(listed.number, number);
        assert_eq!(listed.status, AdrStatus::Accepted);

        if number == 11 {
            assert!(
                after.contains("the platform team"),
                "0011: consulted value must survive"
            );
            assert!(
                !after.contains("consulted:\n  - the platform team\n  the platform team")
                    && !after.contains("consulted:\n  - the platform team\n\n  the platform team"),
                "0011: block-scalar consulted must not be orphaned into a list+continuation\n{after}"
            );
        }
        if number == 12 {
            assert!(
                !after.contains("  - alice\n- alice") && !after.contains("  - bob\n- bob"),
                "0012: zero-indent items must not be orphaned beside canonical list\n{after}"
            );
        }
        if number == 13 {
            let alice_count = after.matches("- alice").count();
            assert!(
                alice_count <= 1,
                "0013: consulted values must not duplicate on update (saw {alice_count})\n{after}"
            );
        }
    }
}

#[test]
fn body_patch_preserves_nested_and_tilde_fences() {
    // Nested/mixed fences must not truncate Decision Outcome or eat the real
    // ### Consequences section (PR #311 review 4707905821 #2).
    let (tmp, repo) = corpus_repo();
    let adr_dir = tmp.path().join("doc/adr");

    for (number, fence_marker) in [(15u32, "````md"), (16u32, "~~~md")] {
        let file = adr_dir.join(fixture_path(number).file_name().unwrap());
        let before = fs::read_to_string(&file).unwrap();
        assert!(
            before.contains(fence_marker),
            "{number:04}: fixture missing {fence_marker}"
        );

        let mut adr = repo.get(number).unwrap();
        adr.consequences = "* Updated consequence from patch".into();
        repo.update(
            &adr,
            BodySectionPatch {
                consequences: Some("* Updated consequence from patch".into()),
                ..Default::default()
            },
        )
        .unwrap_or_else(|e| panic!("update({number}) failed: {e}"));

        let after = fs::read_to_string(&file).unwrap();
        assert!(
            after.contains("Example consequences inside"),
            "{number:04}: in-fence sample body must survive\n{after}"
        );
        assert!(
            after.contains("Trailing text after"),
            "{number:04}: text after fence must survive\n{after}"
        );
        assert!(
            after.contains("### Confirmation"),
            "{number:04}: trailing H3 must survive\n{after}"
        );
        assert!(
            after.contains("* Updated consequence from patch"),
            "{number:04}: real ### Consequences should be patched\n{after}"
        );
        if number == 15 {
            assert!(
                after.matches("````").count() >= 2,
                "0015: outer four-backtick fence must still close\n{after}"
            );
        } else {
            assert!(
                after.matches("~~~").count() >= 2,
                "0016: outer tilde fence must still close\n{after}"
            );
        }
    }
}

#[test]
fn doctor_runs_over_the_corpus() {
    // The corpus is deliberately mixed-format, and the MADR ruleset flags the
    // frontmatter files for using plain `## Context` / `## Decision` headings
    // instead of MADR section names. Pin that doctor completes and reports
    // those as errors rather than crashing or going silent; the exact counts
    // are left unpinned so mdbook-lint rule evolution does not break this test.
    let (_tmp, repo) = corpus_repo();
    let report = adrs_core::lint::check_all(&repo).unwrap();
    assert!(
        report.has_errors(),
        "expected MADR section-name findings on the mixed corpus"
    );
}
