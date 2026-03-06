//! Internal helpers for documentation tests.
//!
//! This module provides reusable setup functions for doctests
//! that need a temporary repository. It is hidden from public
//! documentation but available for use in doctests.

use crate::{Repository, Result};
use tempfile::TempDir;

/// Creates a temporary repository in Compatible mode.
///
/// Returns a tuple of (TempDir, Repository). The TempDir must be
/// kept alive for the duration of the test to prevent the temporary
/// directory from being deleted.
///
/// # Errors
///
/// Returns an error if the temporary directory cannot be created
/// or if repository initialization fails.
pub fn temp_repo() -> Result<(TempDir, Repository)> {
    let temp = TempDir::new().map_err(crate::Error::Io)?;
    let repo = Repository::init(temp.path(), None, false)?;
    Ok((temp, repo))
}

/// Creates a temporary repository in NextGen mode.
///
/// Returns a tuple of (TempDir, Repository). The TempDir must be
/// kept alive for the duration of the test.
///
/// # Errors
///
/// Returns an error if the temporary directory cannot be created
/// or if repository initialization fails.
pub fn temp_repo_nextgen() -> Result<(TempDir, Repository)> {
    let temp = TempDir::new().map_err(crate::Error::Io)?;
    let repo = Repository::init(temp.path(), None, true)?;
    Ok((temp, repo))
}
