#![doc(hidden)]

mod completion;
mod fingerprint;
mod info;
mod list;
mod run;

/// Completion request model and completion entry points.
///
/// Re-exports shell completion script output and dynamic completion protocol
/// handling.
pub(crate) use completion::{CompletionRequest, Shell, complete, print_shell_completion};
/// Fingerprint printing action.
pub(crate) use fingerprint::fingerprint_script;
/// Help text rendering action.
pub(crate) use info::print_help;
/// Script listing action without trust labels.
///
/// Used when trust-manifest support is disabled.
#[cfg(not(feature = "trust-manifest"))]
pub(crate) use list::list_scripts;
/// Script listing action with trust labels.
#[cfg(feature = "trust-manifest")]
pub(crate) use list::list_scripts_with_trust_status;
/// Script execution with explicit fingerprint verification.
pub(crate) use run::run_script_with_fingerprint;
/// Script execution with interactive trust workflow.
#[cfg(feature = "trust-manifest")]
pub(crate) use run::run_script_with_trust;
