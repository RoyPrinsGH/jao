#[cfg(feature = "trust-manifest")]
use std::io::{self, IsTerminal, Write};
use std::path::Path;
use std::process::{Command, Stdio};

#[cfg(feature = "trust-manifest")]
use crate::config::models::JaoContext;
use crate::errors::{JaoError, JaoResult};
#[cfg(feature = "trust-manifest")]
use crate::trust;
#[cfg(feature = "trust-manifest")]
use crate::trust::models::ScriptTrustState;

#[cfg(feature = "trust-manifest")]
pub fn run_script_with_trust(script_path: impl AsRef<Path>, context: &mut JaoContext) -> JaoResult<()> {
    let canonical_path = std::fs::canonicalize(&script_path)?;

    let trust_state = trust::manifest::get_script_trust(&canonical_path, &context.trusted_manifest)?;

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
        let answer = answer.trim().to_ascii_lowercase();

        if answer == "y" || answer == "yes" {
            trust::manifest::write_script_trust_record(&canonical_path, context)?;
        } else {
            return Err(JaoError::ScriptNotTrusted { path: canonical_path });
        }
    }

    execute_script(script_path)
}

pub fn run_script_with_fingerprint(script_path: impl AsRef<Path>, required_fingerprint: &str) -> JaoResult<()> {
    let required_fingerprint = normalize_required_fingerprint(required_fingerprint)?;

    let (canonical_path, actual_fingerprint) = trust::fingerprint::fingerprint_file(&script_path)?;

    if actual_fingerprint != required_fingerprint {
        return Err(JaoError::FingerprintMismatch {
            path: canonical_path,
            expected: required_fingerprint,
            actual: actual_fingerprint,
        });
    }

    execute_script(script_path)
}

fn normalize_required_fingerprint(fingerprint: &str) -> JaoResult<String> {
    let normalized = fingerprint.trim().to_ascii_lowercase();
    let is_valid = normalized.len() == 64 && normalized.chars().all(|ch| ch.is_ascii_hexdigit());

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

    let script_dir = script_path.parent().ok_or_else(|| JaoError::ScriptHasNoParent {
        path: script_path.to_path_buf(),
    })?;

    let script_file = script_path.file_name().ok_or_else(|| JaoError::ScriptHasNoFileName {
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

#[cfg(unix)]
fn is_executable(path: &Path) -> JaoResult<bool> {
    use std::os::unix::fs::PermissionsExt;

    let metadata = std::fs::metadata(path)?;
    Ok(metadata.permissions().mode() & 0o111 != 0)
}

#[cfg(unix)]
fn parse_shebang(path: &Path) -> JaoResult<Option<(String, Vec<String>)>> {
    use std::io::BufRead;

    let file = std::fs::File::open(path)?;
    let mut reader = std::io::BufReader::new(file);
    let mut first_line = String::new();
    reader.read_line(&mut first_line)?;

    if !first_line.starts_with("#!") {
        return Ok(None);
    }

    let shebang = first_line[2..].trim();
    if shebang.is_empty() {
        return Ok(None);
    }

    let mut parts = shebang.split_whitespace();
    let Some(interpreter) = parts.next() else {
        return Ok(None);
    };

    Ok(Some((interpreter.to_string(), parts.map(ToString::to_string).collect())))
}
