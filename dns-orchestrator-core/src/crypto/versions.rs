//! Encryption version configuration.
//!
//! Maps file format versions to encryption parameters.
//!
//! Design principles:
//! - File versions do not expose crypto parameters directly; parameters are defined in code.
//! - Version 1: PBKDF2-HMAC-SHA256, 100,000 iterations
//! - Version 2: PBKDF2-HMAC-SHA256, 600,000 iterations (OWASP 2023 recommendation)
//! - Future versions may switch algorithms (for example Argon2)

/// Version 1: PBKDF2-HMAC-SHA256, 100,000 iterations
const VERSION_1_ITERATIONS: u32 = 100_000;

/// Version 2: PBKDF2-HMAC-SHA256, 600,000 iterations (OWASP 2023 recommendation).
const VERSION_2_ITERATIONS: u32 = 600_000;

/// Current file format version.
///
/// Change this constant to switch versions; iteration count is derived automatically.
pub const CURRENT_FILE_VERSION: u32 = 2;

/// Returns the iteration count of the current version (resolved at compile time).
///
/// Derived from `CURRENT_FILE_VERSION` so encryption/decryption use the same parameters.
///
/// # Panics
/// Panics at compile time if `CURRENT_FILE_VERSION` does not map to a known iteration count.
/// This is intentional: the const fn is evaluated at compile time, so an invalid version
/// will cause a build failure rather than a runtime error.
#[allow(clippy::panic)]
pub const fn get_current_iterations() -> u32 {
    match get_pbkdf2_iterations(CURRENT_FILE_VERSION) {
        Some(iterations) => iterations,
        None => panic!("Invalid CURRENT_FILE_VERSION"),
    }
}

/// Returns the PBKDF2 iteration count for a file version.
///
/// # Arguments
/// * `version` - File version number.
///
/// # Returns
/// - `Some(iterations)` - Iteration count for the version.
/// - `None` - Unsupported version.
pub const fn get_pbkdf2_iterations(version: u32) -> Option<u32> {
    match version {
        1 => Some(VERSION_1_ITERATIONS),
        2 => Some(VERSION_2_ITERATIONS),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_1_iterations() {
        assert_eq!(get_pbkdf2_iterations(1), Some(100_000));
    }

    #[test]
    fn version_2_iterations() {
        assert_eq!(get_pbkdf2_iterations(2), Some(600_000));
    }

    #[test]
    fn unknown_version_returns_none() {
        assert_eq!(get_pbkdf2_iterations(0), None);
        assert_eq!(get_pbkdf2_iterations(99), None);
    }

    #[test]
    fn current_version_is_2() {
        assert_eq!(CURRENT_FILE_VERSION, 2);
    }
}
