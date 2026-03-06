# Linting & Validation

<!-- toc -->

The lint module provides validation and health checks for ADRs and repositories.

## Basic Usage

```rust
use adrs_core::{Repository, lint_all, lint_adr, check_repository, IssueSeverity};

let repo = Repository::open(".")?;

// Lint all ADRs
let report = lint_all(&repo)?;
for issue in report.issues {
    match issue.severity {
        IssueSeverity::Error => eprintln!("Error: {}", issue.message),
        IssueSeverity::Warning => eprintln!("Warning: {}", issue.message),
    }
}

// Repository-level checks
let report = check_repository(&repo)?;
```

## Functions

### lint_adr

Validate a single ADR.

```rust
use adrs_core::{Adr, lint_adr};

let adr = Adr::new(1, "My Decision");
let report = lint_adr(&adr);

if report.has_errors() {
    println!("ADR has errors");
}
```

### lint_all

Lint all ADRs in a repository.

```rust
use adrs_core::{Repository, lint_all};

let repo = Repository::open(".")?;
let report = lint_all(&repo)?;

println!("Found {} issues", report.issues.len());
```

### check_repository

Repository-level checks (broken links, numbering gaps, etc.).

```rust
use adrs_core::{Repository, check_repository};

let repo = Repository::open(".")?;
let report = check_repository(&repo)?;
```

### check_all

Combined ADR and repository checks.

```rust
use adrs_core::{Repository, check_all};

let repo = Repository::open(".")?;
let report = check_all(&repo)?;
```

## Types

### LintReport

```rust
pub struct LintReport {
    pub issues: Vec<Issue>,
}

impl LintReport {
    pub fn has_errors(&self) -> bool;
    pub fn has_warnings(&self) -> bool;
    pub fn error_count(&self) -> usize;
    pub fn warning_count(&self) -> usize;
}
```

### Issue

```rust
pub struct Issue {
    pub severity: IssueSeverity,
    pub message: String,
    pub adr_number: Option<u32>,
    pub path: Option<PathBuf>,
    pub line: Option<usize>,
}
```

### IssueSeverity

```rust
pub enum IssueSeverity {
    Error,    // Must be fixed
    Warning,  // Should be reviewed
}
```

## Checks Performed

### ADR-Level Checks

| Check | Severity | Description |
|-------|----------|-------------|
| Missing title | Error | ADR must have a title |
| Missing status | Error | ADR must have a status |
| Empty context | Warning | Context section is empty |
| Empty decision | Warning | Decision section is empty |
| Empty consequences | Warning | Consequences section is empty |
| Invalid frontmatter | Error | YAML frontmatter is malformed |

### Repository-Level Checks

| Check | Severity | Description |
|-------|----------|-------------|
| Broken links | Error | Link target doesn't exist |
| Numbering gaps | Warning | Missing numbers in sequence |
| Duplicate numbers | Error | Multiple ADRs with same number |
| Orphaned files | Warning | ADR files not matching pattern |

## Example: CI Integration

```rust
use adrs_core::{Repository, check_all};
use std::process::ExitCode;

fn main() -> ExitCode {
    let repo = match Repository::open(".") {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Failed to open repository: {}", e);
            return ExitCode::FAILURE;
        }
    };

    let report = match check_all(&repo) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Failed to run checks: {}", e);
            return ExitCode::FAILURE;
        }
    };

    for issue in &report.issues {
        let prefix = match issue.severity {
            IssueSeverity::Error => "ERROR",
            IssueSeverity::Warning => "WARN",
        };
        eprintln!("[{}] {}", prefix, issue.message);
    }

    if report.has_errors() {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
```

## See Also

- [CLI doctor command](../../users/commands/doctor.md) - User-facing health checks
- [Error Handling](./errors.md) - Error types
