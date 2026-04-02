use std::ffi::OsStr;
#[cfg(feature = "trust-manifest")]
use std::io::{self, IsTerminal, Write};
use std::path::Path;
use std::process::{Command, Stdio};

#[cfg(feature = "trust-manifest")]
use crate::config::JaoConfig;
#[cfg(unix)]
use crate::platform::unix::{is_executable, parse_shebang};
#[cfg(feature = "trust-manifest")]
use crate::trust::manifest::{ScriptTrustState, TrustedManifest};
use crate::{JaoError, JaoResult, trust};

#[cfg(feature = "trust-manifest")]
/// Runs a script under trust-manifest policy.
///
/// Unknown or modified scripts require interactive confirmation unless stdin
/// and stdout are non-terminal, in which case this returns an error.
///
/// On acceptance, this updates the persisted trust manifest before executing
/// the script from its own directory.
pub(crate) fn run_script_with_trust(script_path: impl AsRef<Path>, config: &JaoConfig, manifest: &mut TrustedManifest) -> JaoResult<()> {
    let canonical_path = std::fs::canonicalize(&script_path)?;

    let trust_state = trust::manifest::determine_script_trust_state(&canonical_path, manifest)?;

    if trust_state != ScriptTrustState::Trusted {
        if !(io::stdin().is_terminal() && io::stdout().is_terminal()) {
            return Err(JaoError::UnknownScriptNonInteractive { path: canonical_path });
        }

        match trust_state {
            ScriptTrustState::Unknown => {
                eprintln!("warning: script trust is unknown: {}", canonical_path.display());
                eprint!("trust and run? [y/N]: ");
            }
            ScriptTrustState::Modified => {
                eprintln!("warning: script has been modified since last trust: {}", canonical_path.display());
                eprint!("re-trust and run? [y/N]: ");
            }
            ScriptTrustState::Trusted => {}
        }

        io::stderr().flush()?;

        let mut answer = String::new();
        io::stdin().read_line(&mut answer)?;
        let answer_str = answer.trim();

        if answer_str.eq_ignore_ascii_case("y") || answer_str.eq_ignore_ascii_case("yes") {
            trust::manifest::write_script_trust_record(&canonical_path, &config.trustfile, manifest)?;
        } else {
            return Err(JaoError::ScriptNotTrusted { path: canonical_path });
        }
    }

    execute_script(script_path)
}

/// Runs a script only when its current fingerprint matches `required_fingerprint`.
///
/// The required fingerprint must be a 64-character hexadecimal SHA-256 digest.
///
/// Fingerprint comparison uses the same canonical-path+contents hashing as
/// trust-manifest records.
pub(crate) fn run_script_with_fingerprint(script_path: impl AsRef<Path>, required_fingerprint: &OsStr) -> JaoResult<()> {
    let required_fingerprint = required_fingerprint
        .to_str()
        .ok_or(JaoError::InvalidArguments("required fingerprint contains invalid UTF-8"))?;

    let required_fingerprint = normalize_required_fingerprint(required_fingerprint)?;

    let (canonical_path, record) = trust::create_trust_record(&script_path)?;

    if record.fingerprint != required_fingerprint {
        return Err(JaoError::FingerprintMismatch {
            path: canonical_path,
            expected: required_fingerprint,
            actual: record.fingerprint,
        });
    }

    execute_script(script_path)
}

fn normalize_required_fingerprint(fingerprint: &str) -> JaoResult<String> {
    let normalized = fingerprint
        .trim()
        .to_ascii_lowercase();

    let is_valid = normalized.len() == 64
        && normalized
            .chars()
            .all(|ch| ch.is_ascii_hexdigit());

    if is_valid {
        Ok(normalized)
    } else {
        Err(JaoError::InvalidRequiredFingerprint {
            fingerprint: fingerprint.to_string(),
        })
    }
}

fn execute_script(script_path: impl AsRef<Path>) -> JaoResult<()> {
    let script_path = script_path.as_ref();

    let script_dir = script_path
        .parent()
        .ok_or_else(|| JaoError::ScriptHasNoParent {
            path: script_path.to_path_buf(),
        })?;

    let script_file = script_path
        .file_name()
        .ok_or_else(|| JaoError::ScriptHasNoFileName {
            path: script_path.to_path_buf(),
        })?;

    #[cfg(windows)]
    let status = Command::new("cmd")
        .arg("/C")
        .arg(script_file)
        .current_dir(script_dir)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;

    #[cfg(unix)]
    let status = if is_executable(script_path)? {
        Command::new(Path::new(".").join(script_file))
            .current_dir(script_dir)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()?
    } else if let Some((interpreter, interpreter_args)) = parse_shebang(script_path)? {
        Command::new(interpreter)
            .args(interpreter_args)
            .arg(script_file)
            .current_dir(script_dir)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()?
    } else {
        return Err(JaoError::ScriptNotExecutableAndNoShebang {
            path: script_path.to_path_buf(),
        });
    };

    if status.success() {
        Ok(())
    } else {
        Err(JaoError::ScriptFailed { status })
    }
}
