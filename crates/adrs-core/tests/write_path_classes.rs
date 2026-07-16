//! Class-level write-path matrices for metadata and body updates.
//!
//! These pin *classes* of inputs × write APIs, not single string repros:
//! people-YAML shapes under metadata writes, nested/mixed fences under body
//! patches, and Consequences append after a suppressed trailing newline.

use adrs_core::{AdrStatus, BodySectionPatch, LinkKind, Repository};
use std::fs;
use tempfile::TempDir;

fn write_ng_fixture(repo: &Repository, number: u32, content: &str) {
    let name = format!("{number:04}-class-fixture.md");
    fs::write(repo.adr_path().join(name), content).unwrap();
}

fn frontmatter_after_status(content: &str) -> String {
    content
        .split("---\n")
        .nth(1)
        .unwrap_or("")
        .split("\n---")
        .next()
        .unwrap_or("")
        .to_string()
}

/// People-field YAML shapes that must survive metadata writes.
fn people_yaml_cases() -> Vec<(&'static str, &'static str)> {
    vec![
        (
            "block-scalar",
            r#"---
number: 2
title: Block scalar
date: 2024-06-01
status: proposed
consulted: >-
  the platform team
---

## Context

C.

## Decision

D.

## Consequences

X.
"#,
        ),
        (
            "zero-indent",
            r#"---
number: 2
title: Zero indent
date: 2024-06-01
status: proposed
consulted:
- alice
- bob
---

## Context

C.

## Decision

D.

## Consequences

X.
"#,
        ),
        (
            "comment-between",
            r#"---
number: 2
title: Comment between
date: 2024-06-01
status: proposed
consulted:
  # people
  - alice
---

## Context

C.

## Decision

D.

## Consequences

X.
"#,
        ),
        (
            "four-space",
            r#"---
number: 2
title: Four space
date: 2024-06-01
status: proposed
consulted:
    - alice
    - bob
---

## Context

C.

## Decision

D.

## Consequences

X.
"#,
        ),
        (
            "scalar-decision-makers",
            r#"---
number: 2
title: Scalar makers
date: 2024-06-01
status: proposed
decision-makers: alice
---

## Context

C.

## Decision

D.

## Consequences

X.
"#,
        ),
    ]
}

#[test]
fn people_yaml_noop_metadata_byte_identical() {
    for (name, fixture) in people_yaml_cases() {
        let temp = TempDir::new().unwrap();
        let repo = Repository::init(temp.path(), None, true).unwrap();
        write_ng_fixture(&repo, 2, fixture);
        let path = repo.adr_path().join("0002-class-fixture.md");
        let before = fs::read(&path).unwrap();
        let adr = repo.get(2).unwrap();
        repo.update_metadata(&adr).unwrap();
        let after = fs::read(&path).unwrap();
        assert_eq!(
            before, after,
            "{name}: no-op update_metadata must be byte-identical"
        );
    }
}

#[test]
fn people_yaml_set_status_keeps_adr_listable() {
    // Primary surface: adrs status / Repository::set_status.
    for (name, fixture) in people_yaml_cases() {
        let temp = TempDir::new().unwrap();
        let repo = Repository::init(temp.path(), None, true).unwrap();
        write_ng_fixture(&repo, 2, fixture);
        let path = repo.adr_path().join("0002-class-fixture.md");

        repo.set_status(2, AdrStatus::Accepted, None)
            .unwrap_or_else(|e| panic!("{name}: set_status failed: {e}"));

        let after = fs::read_to_string(&path).unwrap();
        let listed = repo.get(2).unwrap_or_else(|e| {
            panic!("{name}: ADR disappeared from list after set_status: {e}\n{after}")
        });
        assert_eq!(listed.status, AdrStatus::Accepted);
        assert!(
            after.contains("status: accepted"),
            "{name}: status line missing\n{after}"
        );

        // Orphaned continuation / duplicated list items are the failure mode.
        if name == "block-scalar" {
            assert!(
                !after.contains("consulted:\n  - the platform team\n  the platform team")
                    && !after.contains("consulted:\n  - the platform team\n\n  the platform team"),
                "{name}: block-scalar continuation orphaned\n{after}"
            );
        }
        if name == "zero-indent" || name == "four-space" {
            let fm = frontmatter_after_status(&after);
            assert!(
                !(fm.contains("  - alice") && fm.contains("\n- alice")),
                "{name}: list items orphaned beside canonical form\n{fm}"
            );
        }
        if name == "comment-between" {
            assert!(
                after.matches("- alice").count() <= 1,
                "{name}: values duplicated\n{after}"
            );
        }
    }
}

#[test]
fn people_yaml_intentional_consulted_change_stays_parseable() {
    // When the caller actually changes consulted, the file must still parse.
    for (name, fixture) in people_yaml_cases() {
        let temp = TempDir::new().unwrap();
        let repo = Repository::init(temp.path(), None, true).unwrap();
        write_ng_fixture(&repo, 2, fixture);
        let path = repo.adr_path().join("0002-class-fixture.md");

        let mut adr = repo.get(2).unwrap();
        adr.set_consulted(vec!["alice".into(), "dave".into()]);
        repo.update_metadata(&adr)
            .unwrap_or_else(|e| panic!("{name}: update_metadata failed: {e}"));

        let after = fs::read_to_string(&path).unwrap();
        repo.get(2).unwrap_or_else(|e| {
            panic!("{name}: ADR unreadable after intentional consulted change: {e}\n{after}")
        });
        // Classic regex splice failure: canonical block written, old items left behind.
        assert!(
            !(after.contains("  - alice\n  - dave") && after.contains("\n- alice\n- bob")),
            "{name}: must not leave orphaned zero-indent items beside rewrite\n{after}"
        );
        assert!(
            !after.contains("  - dave\n- alice"),
            "{name}: orphaned list items after splice\n{after}"
        );
    }
}

fn fence_patch_cases() -> Vec<(&'static str, &'static str)> {
    vec![
        (
            "nested-backticks",
            r#"---
number: 2
title: Nested backticks
date: 2024-06-01
status: accepted
---

## Decision Outcome

Chosen: X.

````md
```
## Consequences

Example nested.
```
````

Trailing after nested.

### Consequences

* Old good

### Confirmation

Keep me.
"#,
        ),
        (
            "tilde-wraps-backticks",
            r#"---
number: 2
title: Tilde wraps backticks
date: 2024-06-01
status: accepted
---

## Decision Outcome

Chosen: Y.

~~~md
```markdown
## Consequences

Example tilde-wrapped.
```
~~~

Trailing after tilde.

### Consequences

* Old good

### Confirmation

Keep me.
"#,
        ),
        (
            "backticks-wrap-tilde",
            r#"---
number: 2
title: Backticks wrap tilde
date: 2024-06-01
status: accepted
---

## Decision Outcome

Chosen: Z.

```md
~~~
## Consequences

Example backtick-wrapped tilde.
~~~
```

Trailing after mixed.

### Consequences

* Old good

### Confirmation

Keep me.
"#,
        ),
        (
            "four-space-indented-fence-lookalike",
            // At 4 spaces, CommonMark treats ``` as indented code, not a fence.
            // Heading lookalikes inside must not toggle fence state.
            r#"---
number: 2
title: Indented code lookalike
date: 2024-06-01
status: accepted
---

## Decision Outcome

Chosen: W.

    ```
    ## Consequences
    not a fence
    ```

### Consequences

* Old good

### Confirmation

Keep me.
"#,
        ),
    ]
}

#[test]
fn fence_forms_consequences_patch_preserves_samples() {
    for (name, fixture) in fence_patch_cases() {
        let temp = TempDir::new().unwrap();
        let repo = Repository::init(temp.path(), None, true).unwrap();
        write_ng_fixture(&repo, 2, fixture);
        let path = repo.adr_path().join("0002-class-fixture.md");

        let mut adr = repo.get(2).unwrap();
        adr.consequences = "* New good.".into();
        repo.update(
            &adr,
            BodySectionPatch {
                consequences: Some("* New good.".into()),
                ..Default::default()
            },
        )
        .unwrap_or_else(|e| panic!("{name}: update failed: {e}"));

        let after = fs::read_to_string(&path).unwrap();
        assert!(
            after.contains("### Confirmation") && after.contains("Keep me."),
            "{name}: trailing H3 destroyed\n{after}"
        );
        assert!(
            after.contains("* New good."),
            "{name}: real consequences not patched\n{after}"
        );
        assert!(
            after.contains("Trailing after") || name == "four-space-indented-fence-lookalike",
            "{name}: trailing text after fence destroyed\n{after}"
        );
        match name {
            "nested-backticks" => assert!(
                after.matches("````").count() >= 2,
                "{name}: outer fence not closed\n{after}"
            ),
            "tilde-wraps-backticks" => assert!(
                after.matches("~~~").count() >= 2,
                "{name}: outer tilde fence not closed\n{after}"
            ),
            "backticks-wrap-tilde" => assert!(
                after.contains("Example backtick-wrapped tilde."),
                "{name}: inner sample lost\n{after}"
            ),
            "four-space-indented-fence-lookalike" => assert!(
                after.contains("not a fence"),
                "{name}: indented sample lost\n{after}"
            ),
            _ => {}
        }
    }
}

#[test]
fn append_consequences_after_no_trailing_newline_is_not_glued() {
    let temp = TempDir::new().unwrap();
    let repo = Repository::init(temp.path(), None, true).unwrap();

    // No trailing newline; Decision Outcome has no ### Consequences yet.
    let content = "---\nnumber: 2\ntitle: No trailing newline append\ndate: 2024-06-01\nstatus: accepted\n---\n\n## Decision Outcome\n\nChosen option only.";
    let path = repo.adr_path().join("0002-class-fixture.md");
    fs::write(&path, content).unwrap();
    assert_ne!(fs::read(&path).unwrap().last(), Some(&b'\n'));

    let mut adr = repo.get(2).unwrap();
    adr.consequences = "* Appended.".into();
    repo.update(
        &adr,
        BodySectionPatch {
            consequences: Some("* Appended.".into()),
            ..Default::default()
        },
    )
    .unwrap();

    let after = fs::read_to_string(&path).unwrap();
    assert!(
        !after.contains("only.### Consequences") && !after.contains("only.###"),
        "### Consequences must not glue onto prior line\n{after}"
    );
    assert!(
        after.contains("\n### Consequences\n"),
        "### Consequences must be on its own line\n{after}"
    );
    assert!(after.contains("* Appended."));
}

#[test]
fn create_then_append_consequences_not_glued() {
    // repo.create() often produces no trailing newline.
    let temp = TempDir::new().unwrap();
    let repo = Repository::init(temp.path(), None, true).unwrap();
    let mut adr = adrs_core::Adr::new(2, "Create append");
    adr.status = AdrStatus::Accepted;
    adr.decision = "Chosen option only.".into();
    let path = repo.create(&adr).unwrap();
    let raw = fs::read(&path).unwrap();
    // Precondition worth knowing; do not fail the test solely on it.
    let _ends_without_nl = raw.last() != Some(&b'\n');

    // Force MADR-style Decision Outcome without ### Consequences, no trailing NL.
    let forced = "---\nnumber: 2\ntitle: Create append\ndate: 2024-06-01\nstatus: accepted\n---\n\n## Decision Outcome\n\nChosen option only.";
    fs::write(&path, forced).unwrap();
    let mut adr = repo.get(2).unwrap();
    adr.consequences = "* From create path.".into();
    repo.update(
        &adr,
        BodySectionPatch {
            consequences: Some("* From create path.".into()),
            ..Default::default()
        },
    )
    .unwrap();

    let after = fs::read_to_string(&path).unwrap();
    assert!(
        !after.contains("only.### Consequences"),
        "create/append path glued heading\n{after}"
    );
    assert!(
        after.contains("\n### Consequences\n"),
        "expected standalone ### Consequences\n{after}"
    );
}

#[test]
fn madr_consequences_h2_before_decision_outcome_is_not_duplicated() {
    // When ## Consequences appears before ## Decision Outcome, a consequences
    // patch must update that H2 once and must not also inject ### Consequences.
    let fixture = r#"---
number: 2
title: Consequences before Decision Outcome
date: 2024-06-01
status: accepted
---

## Context and Problem Statement

Context.

## Consequences

* Old consequence

## Decision Outcome

Chosen option: X.

### Confirmation

Keep me.
"#;
    let temp = TempDir::new().unwrap();
    let repo = Repository::init(temp.path(), None, true).unwrap();
    write_ng_fixture(&repo, 2, fixture);
    let path = repo.adr_path().join("0002-class-fixture.md");

    repo.update(
        &repo.get(2).unwrap(),
        BodySectionPatch {
            consequences: Some("* New consequence".into()),
            ..Default::default()
        },
    )
    .unwrap();

    let after = fs::read_to_string(&path).unwrap();
    assert!(
        after.contains("* New consequence"),
        "existing ## Consequences H2 must be patched\n{after}"
    );
    assert!(
        !after.contains("### Consequences"),
        "must not inject ### Consequences when ## Consequences already exists\n{after}"
    );
    assert_eq!(
        after.matches("* New consequence").count(),
        1,
        "consequence text must appear once\n{after}"
    );
    assert!(
        after.contains("Chosen option: X.") && after.contains("### Confirmation"),
        "Decision Outcome intro and Confirmation must survive\n{after}"
    );
    assert!(
        !after.contains("* Old consequence"),
        "old H2 body should be replaced\n{after}"
    );
}

#[test]
fn set_status_preserves_unknown_frontmatter_keys() {
    // Progressive baseline: unmanaged frontmatter keys must survive set_status
    // when the YAML Mapping is re-emitted.
    let fixture = r#"---
number: 2
title: Unknown keys
date: 2024-06-01
status: proposed
custom-meta: keep-me
extra-list:
  - one
  - two
---

## Context

C.

## Decision

D.

## Consequences

X.
"#;
    let temp = TempDir::new().unwrap();
    let repo = Repository::init(temp.path(), None, true).unwrap();
    write_ng_fixture(&repo, 2, fixture);
    let path = repo.adr_path().join("0002-class-fixture.md");

    repo.set_status(2, AdrStatus::Accepted, None).unwrap();

    let after = fs::read_to_string(&path).unwrap();
    assert!(
        after.contains("custom-meta: keep-me") || after.contains("custom-meta:keep-me"),
        "unmanaged scalar key must survive set_status\n{after}"
    );
    assert!(
        after.contains("extra-list:") && after.contains("one") && after.contains("two"),
        "unmanaged list key must survive set_status\n{after}"
    );
    assert!(after.contains("status: accepted"));
    repo.get(2).unwrap();
}

#[test]
fn set_status_scalar_makers_does_not_corrupt_block_scalar_consulted() {
    // Cross-field: scalar decision-makers + block-scalar consulted on status write.
    let fixture = r#"---
number: 2
title: Cross-field people
date: 2024-06-01
status: proposed
decision-makers: alice
consulted: >-
  the platform team
---

## Context

C.

## Decision

D.

## Consequences

X.
"#;
    let temp = TempDir::new().unwrap();
    let repo = Repository::init(temp.path(), None, true).unwrap();
    write_ng_fixture(&repo, 2, fixture);
    let path = repo.adr_path().join("0002-class-fixture.md");

    repo.set_status(2, AdrStatus::Accepted, None)
        .unwrap_or_else(|e| panic!("set_status failed: {e}"));

    let after = fs::read_to_string(&path).unwrap();
    let listed = repo
        .get(2)
        .unwrap_or_else(|e| panic!("ADR disappeared after set_status (cross-field): {e}\n{after}"));
    assert_eq!(listed.status, AdrStatus::Accepted);
    assert!(
        after.contains("the platform team"),
        "consulted value lost\n{after}"
    );
    assert!(
        !after.contains("consulted:\n  - the platform team\n  the platform team")
            && !after.contains("consulted:\n  - the platform team\n\n  the platform team"),
        "block-scalar consulted must not be orphaned beside list rewrite\n{after}"
    );
}

#[test]
fn add_link_preserves_block_scalar_consulted() {
    // link() on block-scalar people YAML must not drop the ADR from list.
    let source = r#"---
number: 2
title: Link source
date: 2024-06-01
status: proposed
consulted: >-
  the platform team
---

## Context

C.

## Decision

D.

## Consequences

X.
"#;
    let target = r#"---
number: 3
title: Link target
date: 2024-06-01
status: accepted
---

## Context

C.

## Decision

D.

## Consequences

X.
"#;
    let temp = TempDir::new().unwrap();
    let repo = Repository::init(temp.path(), None, true).unwrap();
    write_ng_fixture(&repo, 2, source);
    write_ng_fixture(&repo, 3, target);
    let path = repo.adr_path().join("0002-class-fixture.md");

    repo.link(2, 3, LinkKind::Amends, LinkKind::AmendedBy)
        .unwrap_or_else(|e| panic!("link failed: {e}"));

    let after = fs::read_to_string(&path).unwrap();
    let listed = repo
        .get(2)
        .unwrap_or_else(|e| panic!("ADR disappeared after link: {e}\n{after}"));
    assert!(
        listed.links.iter().any(|l| l.target == 3),
        "link not recorded\n{after}"
    );
    assert!(
        after.contains("the platform team"),
        "consulted value lost after link\n{after}"
    );
    assert!(
        !after.contains("consulted:\n  - the platform team\n  the platform team")
            && !after.contains("consulted:\n  - the platform team\n\n  the platform team"),
        "block-scalar consulted must not be orphaned after link\n{after}"
    );
}

#[test]
fn noop_metadata_preserves_zero_indent_tags_or_links() {
    // Pins tags/links zero-indent lists (same YAML class as people fields).
    let fixture = r#"---
number: 2
title: Zero indent tags links
date: 2024-06-01
status: proposed
tags:
- alpha
- beta
links:
- target: 3
  kind: amends
---

## Context

C.

## Decision

D.

## Consequences

X.
"#;
    let temp = TempDir::new().unwrap();
    let repo = Repository::init(temp.path(), None, true).unwrap();
    write_ng_fixture(&repo, 2, fixture);
    let path = repo.adr_path().join("0002-class-fixture.md");

    let adr = repo.get(2).unwrap();
    repo.update_metadata(&adr)
        .unwrap_or_else(|e| panic!("update_metadata failed: {e}"));

    let after = fs::read_to_string(&path).unwrap();
    repo.get(2)
        .unwrap_or_else(|e| panic!("ADR unreadable after noop tags/links: {e}\n{after}"));
    let fm = frontmatter_after_status(&after);
    assert!(
        !(fm.contains("  - alpha") && fm.contains("\n- alpha")),
        "tags must not leave orphaned zero-indent items beside rewrite\n{fm}"
    );
    assert!(
        after.contains("alpha") && after.contains("beta"),
        "tag values lost\n{after}"
    );
    assert!(
        after.contains("target: 3") && after.contains("kind: amends"),
        "link lost after noop metadata\n{after}"
    );
}

#[test]
fn baseline_body_only_preserves_non_canonical_status() {
    // Progressive baseline — must stay green (finding 4).
    let temp = TempDir::new().unwrap();
    let repo = Repository::init(temp.path(), None, false).unwrap();
    let content = r#"# 2. Non-canonical status

Date: 2026-01-15

## Status

Approved by the architecture board 2024-06-01

## Context

Original context.

## Decision

D.

## Consequences

C.
"#;
    let path = repo.adr_path().join("0002-status-prose.md");
    fs::write(&path, content).unwrap();
    repo.update(
        &repo.get(2).unwrap(),
        BodySectionPatch {
            context: Some("Updated context.".into()),
            ..Default::default()
        },
    )
    .unwrap();
    let after = fs::read_to_string(&path).unwrap();
    assert!(after.contains("Approved by the architecture board"));
    assert!(after.contains("Updated context."));
}

#[test]
fn baseline_simple_fence_consequences_patch() {
    // Progressive baseline — must stay green (exact fence repro).
    let temp = TempDir::new().unwrap();
    let repo = Repository::init(temp.path(), None, true).unwrap();
    let content = r#"---
number: 2
title: Simple fence
date: 2024-06-01
status: accepted
---

## Decision Outcome

Chosen.

```markdown
## Consequences

Inside.
```

Trailing.

### Consequences

* Old

### Confirmation

Keep.
"#;
    let path = repo.adr_path().join("0002-simple-fence.md");
    fs::write(&path, content).unwrap();
    let mut adr = repo.get(2).unwrap();
    adr.consequences = "* New".into();
    repo.update(
        &adr,
        BodySectionPatch {
            consequences: Some("* New".into()),
            ..Default::default()
        },
    )
    .unwrap();
    let after = fs::read_to_string(&path).unwrap();
    assert!(after.contains("Inside."));
    assert!(after.contains("### Confirmation"));
    assert!(after.contains("* New"));
}

#[test]
fn baseline_decision_patch_no_trailing_newline() {
    // Progressive baseline — must stay green (exact append_lines repro).
    let temp = TempDir::new().unwrap();
    let repo = Repository::init(temp.path(), None, false).unwrap();
    let content = "# 2. Compact\n\nDate: 2026-01-15\n\n## Status\n\nAccepted\n\n## Context\n\nOld context.\n\n## Decision\n\nOld decision.\n\n## Consequences\n\nOld consequences.";
    let path = repo.adr_path().join("0002-compact.md");
    fs::write(&path, content).unwrap();
    repo.update(
        &repo.get(2).unwrap(),
        BodySectionPatch {
            decision: Some("New decision.".into()),
            ..Default::default()
        },
    )
    .unwrap();
    let after = fs::read_to_string(&path).unwrap();
    assert!(!after.contains("Old context.## Decision"));
    assert!(after.contains("New decision."));
}
