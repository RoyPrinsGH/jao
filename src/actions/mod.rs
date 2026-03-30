#![doc(hidden)]

mod completion;
mod fingerprint;
mod info;
mod list;
mod run;

pub(crate) use completion::{CompletionRequest, Shell, complete, print_shell_completion};
pub(crate) use fingerprint::fingerprint_script;
pub(crate) use info::print_help;
#[cfg(not(feature = "trust-manifest"))]
pub(crate) use list::list_scripts;
#[cfg(feature = "trust-manifest")]
pub(crate) use list::list_scripts_with_trust_status;
pub(crate) use run::run_script_with_fingerprint;
#[cfg(feature = "trust-manifest")]
pub(crate) use run::run_script_with_trust;
