//! XDG Base Directory Specification helpers.
//!
//! This module provides platform-aware path resolution for configuration
//! files, following the XDG Base Directory Specification on Unix systems.
//!
//! # Overview
//!
//! | Platform | Config Directory |
//! |----------|------------------|
//! | Linux | `$XDG_CONFIG_HOME/adrs` or `~/.config/adrs` |
//! | macOS | `~/Library/Application Support/adrs` |
//! | Windows | `%APPDATA%\adrs` |
//!
//! # Environment Variables
//!
//! | Variable | Purpose |
//! |----------|---------|
//! | `ADRS_CONFIG_DIR` | Explicit override for config directory |
//! | `XDG_CONFIG_HOME` | XDG standard config home (Unix) |
//! | `GIT_CONFIG_GLOBAL` | Override for global gitconfig |
//! | `GIT_CONFIG_SYSTEM` | Override for system gitconfig |

use std::path::PathBuf;

use crate::{Error, Result};

/// Resolve the user's global config directory.
///
/// This is where user-level adrs configuration and templates are stored.
/// The directory may not exist; callers should create it if needed.
///
/// # Resolution Order
///
/// 1. `ADRS_CONFIG_DIR` env var (if set) - explicit override
/// 2. `XDG_CONFIG_HOME/adrs/` (if `XDG_CONFIG_HOME` set) - XDG compliance
/// 3. `~/.config/adrs/` (default on Unix)
/// 4. `%APPDATA%/adrs/` (default on Windows)
///
/// # Returns
///
/// Returns `Some(PathBuf)` with the config directory path.
/// Returns `None` if the home/config directory cannot be determined.
///
/// # Examples
///
/// ```rust,ignore
/// use adrs_core::config::xdg::global_config_dir;
///
/// if let Some(dir) = global_config_dir() {
///     let config_file = dir.join("config.toml");
///     // Load user config...
/// }
/// ```
///
/// # Platform Notes
///
/// - Unix: Follows XDG Base Directory Specification
/// - Windows: Uses `%APPDATA%` (typically `C:\Users\<user>\AppData\Roaming`)
/// - macOS: Uses `~/Library/Application Support` via `dirs` crate
pub fn global_config_dir() -> Option<PathBuf> {
    // 1. Explicit override
    if let Ok(dir) = std::env::var("ADRS_CONFIG_DIR")
        && !dir.trim().is_empty()
    {
        return Some(PathBuf::from(dir));
    }

    // 2. XDG on Unix
    #[cfg(unix)]
    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME")
        && !xdg.trim().is_empty()
    {
        return Some(PathBuf::from(xdg).join("adrs"));
    }

    // 3/4. Platform defaults
    dirs::config_dir().map(|d| d.join("adrs"))
}

/// Resolve the global gitconfig path.
///
/// # Returns
///
/// Returns `Ok(PathBuf)` with the global gitconfig path.
/// Returns `Err` if the home directory cannot be determined.
///
/// # Environment
///
/// Respects `GIT_CONFIG_GLOBAL` env var, falls back to `~/.gitconfig`.
///
/// # Errors
///
/// Returns an error if the home directory cannot be determined and
/// `GIT_CONFIG_GLOBAL` is not set.
///
/// # Examples
///
/// ```rust,ignore
/// use adrs_core::config::xdg::global_gitconfig_path;
///
/// let path = global_gitconfig_path()?;
/// // ~/.gitconfig or $GIT_CONFIG_GLOBAL
/// ```
pub fn global_gitconfig_path() -> Result<PathBuf> {
    if let Ok(path) = std::env::var("GIT_CONFIG_GLOBAL")
        && !path.trim().is_empty()
    {
        return Ok(PathBuf::from(path));
    }

    dirs::home_dir()
        .map(|h| h.join(".gitconfig"))
        .ok_or_else(|| Error::ConfigError("Cannot determine home directory".into()))
}

/// Resolve the system gitconfig path.
///
/// # Returns
///
/// Returns the system gitconfig path. This path may not exist.
///
/// # Environment
///
/// Respects `GIT_CONFIG_SYSTEM` env var, falls back to `/etc/gitconfig`.
///
/// # Platform Notes
///
/// - Unix: `/etc/gitconfig`
/// - Windows: `C:\ProgramData\Git\config` (typically)
///
/// # Examples
///
/// ```rust,ignore
/// use adrs_core::config::xdg::system_gitconfig_path;
///
/// let path = system_gitconfig_path();
/// // /etc/gitconfig or $GIT_CONFIG_SYSTEM
/// ```
pub fn system_gitconfig_path() -> PathBuf {
    if let Ok(path) = std::env::var("GIT_CONFIG_SYSTEM")
        && !path.trim().is_empty()
    {
        return PathBuf::from(path);
    }

    #[cfg(unix)]
    {
        PathBuf::from("/etc/gitconfig")
    }

    #[cfg(windows)]
    {
        PathBuf::from("C:\\ProgramData\\Git\\config")
    }

    #[cfg(not(any(unix, windows)))]
    {
        PathBuf::from("/etc/gitconfig")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static ENV_MUTEX: Mutex<()> = Mutex::new(());

    fn with_env<F, T>(vars: &[(&str, Option<&str>)], f: F) -> T
    where
        F: FnOnce() -> T,
    {
        let _guard = ENV_MUTEX.lock().unwrap();

        let originals: Vec<_> = vars
            .iter()
            .map(|(k, _)| (*k, std::env::var(*k).ok()))
            .collect();

        // SAFETY: We hold a mutex to ensure single-threaded access to env vars
        for (key, value) in vars {
            match value {
                Some(v) => unsafe { std::env::set_var(key, v) },
                None => unsafe { std::env::remove_var(key) },
            }
        }

        let result = f();

        // SAFETY: We hold a mutex to ensure single-threaded access to env vars
        for (key, original) in originals {
            match original {
                Some(v) => unsafe { std::env::set_var(key, v) },
                None => unsafe { std::env::remove_var(key) },
            }
        }

        result
    }

    #[test]
    fn adrs_config_dir_overrides_all() {
        with_env(
            &[
                ("ADRS_CONFIG_DIR", Some("/custom/adrs")),
                ("XDG_CONFIG_HOME", Some("/xdg/config")),
            ],
            || {
                let dir = global_config_dir();
                assert_eq!(dir, Some(PathBuf::from("/custom/adrs")));
            },
        );
    }

    #[cfg(unix)]
    #[test]
    fn xdg_config_home_used_on_unix() {
        with_env(
            &[
                ("ADRS_CONFIG_DIR", None),
                ("XDG_CONFIG_HOME", Some("/xdg/config")),
            ],
            || {
                let dir = global_config_dir();
                assert_eq!(dir, Some(PathBuf::from("/xdg/config/adrs")));
            },
        );
    }

    #[test]
    fn git_config_global_override() {
        with_env(&[("GIT_CONFIG_GLOBAL", Some("/custom/.gitconfig"))], || {
            let path = global_gitconfig_path().unwrap();
            assert_eq!(path, PathBuf::from("/custom/.gitconfig"));
        });
    }

    #[test]
    fn git_config_system_override() {
        with_env(&[("GIT_CONFIG_SYSTEM", Some("/custom/gitconfig"))], || {
            let path = system_gitconfig_path();
            assert_eq!(path, PathBuf::from("/custom/gitconfig"));
        });
    }

    #[cfg(unix)]
    #[test]
    fn system_gitconfig_default_unix() {
        with_env(&[("GIT_CONFIG_SYSTEM", None)], || {
            let path = system_gitconfig_path();
            assert_eq!(path, PathBuf::from("/etc/gitconfig"));
        });
    }
}
