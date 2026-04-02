#![doc(hidden)]

use std::io::IsTerminal;

use crate::JaoResult;

const BOLD: &str = "\x1b[1m";
const UNDERLINE: &str = "\x1b[4m";
const CYAN: &str = "\x1b[36m";
const GREEN: &str = "\x1b[32m";
const RESET: &str = "\x1b[0m";
const OPTION_COL_WIDTH: usize = 28;
const OPTION_DESC_INDENT: &str = "                              ";

/// Prints CLI help text to stdout.
///
/// Output is styled when stdout is a terminal; otherwise plain text is used.
pub(crate) fn print_help() -> JaoResult<()> {
    if should_style() {
        println!("{BOLD}{CYAN}jao - discover, inspect, and run workspace scripts{RESET}");
        line("  Finds platform scripts recursively and executes them from their own directory.");
        println!();
        section("USAGE");
        line("  jao --list");
        line("  jao --ci --list");
        line("  jao --completions <bash|zsh>");
        line("  jao --fingerprint <SCRIPT_COMMAND>...");
        line("  jao <SCRIPT_COMMAND>...");
        #[cfg(feature = "trust-manifest")]
        line("  jao --ci --require-fingerprint <FINGERPRINT> <SCRIPT_COMMAND>...");
        #[cfg(not(feature = "trust-manifest"))]
        line("  jao --require-fingerprint <FINGERPRINT> <SCRIPT_COMMAND>...");
        println!();

        section("OPTIONS");
        option("  -h, --help", "Show this help screen");
        option("      --list", "List runnable scripts as command names plus their resolved file paths");
        option("      --completions <SHELL>", "Print a bash or zsh completion script");
        option(
            "      --fingerprint <SCRIPT_COMMAND>...",
            "Resolve a script command, then print SHA-256 of canonical path + file contents",
        );
        option("      --ci", "Enable CI mode (non-interactive, no config/manifest I/O)");
        option(
            "      --require-fingerprint <FINGERPRINT>",
            "Require exact script fingerprint before running",
        );
        option("  -V, --version", "Print version");
        println!();

        section("SCRIPT COMMAND INPUT");
        line("  Script file stems still use '.' as command separators.");
        line("  Directories contribute command parts only when they contain a .jaofolder marker.");
        line("  Discovery also respects .gitignore.");
        line("  .jaoignore files are applied recursively and exclude matching directories and scripts.");
        line("  Multi-project example:");
        line("    apps/frontend/scripts/dev.sh with apps/ and frontend/ marked becomes:");
        line("    jao apps frontend dev from the workspace root");
        line("    jao frontend dev from inside apps/");
        line("    jao dev from inside apps/frontend/");
        line("  Matching extension is chosen by OS: .sh on Unix-like systems, .bat on Windows.");
        line("  The script runs with working directory set to the script's folder.");
        line("  Shell completion can suggest script parts dynamically from the current directory.");
        println!();

        section("TRUST BEHAVIOR");
        #[cfg(feature = "trust-manifest")]
        {
            line("  Running a script requires trust.");
            line("  Unknown scripts prompt: trust and run? [y/N]");
            line("  Modified scripts prompt: re-trust and run? [y/N]");
            line("  In non-interactive mode, unknown/modified scripts fail.");
            line("  --list prints trust state labels plus command names and resolved paths.");
            line("  --require-fingerprint can be used in CI mode.");
        }
        #[cfg(not(feature = "trust-manifest"))]
        {
            line("  This build has trust-manifest disabled.");
            line("  All runs require --require-fingerprint <FINGERPRINT>.");
            line("  Runs are always non-interactive and never write trust state.");
            line("  --list prints command names and resolved paths.");
        }
        line("  --ci skips config/manifest creation and never prompts.");
        #[cfg(feature = "trust-manifest")]
        line("  --ci run requires --require-fingerprint <FINGERPRINT>.");
        #[cfg(not(feature = "trust-manifest"))]
        line("  --ci uses the same fingerprint-required run policy.");
        line("  --ci --list prints command names and resolved paths (no trust labels).");
        println!();

        section("EXAMPLES");
        example("  jao check");
        line("    Run check.sh / check.bat if found.");
        example("  jao test integration");
        line("    Run test.integration.sh / .bat if found.");
        example("  jao db reset local");
        line("    Run db.reset.local.sh / .bat if found.");
        example("  source <(jao --completions bash)");
        line("    Install bash completion for the current shell.");
        example("  jao m<TAB>");
        line("    Dynamic completion can expand that to myapp, then deeper parts like backend or build.");
        example("  jao --list");
        #[cfg(feature = "trust-manifest")]
        line("    Output includes trust state, command name, and resolved path.");
        #[cfg(not(feature = "trust-manifest"))]
        line("    Output includes command name and resolved path.");
        example("  jao apps backend build");
        line("    Example of using .jaofolder in a multi-project repo.");
        example("  jao --ci --require-fingerprint <FINGERPRINT> db reset local");
        line("    Run only if the resolved script fingerprint matches exactly.");
        #[cfg(feature = "trust-manifest")]
        example("  jao --fingerprint db reset local");
        #[cfg(not(feature = "trust-manifest"))]
        example("  jao --require-fingerprint <FINGERPRINT> db reset local");
        #[cfg(feature = "trust-manifest")]
        line("    Print the fingerprint you can later require in CI.");
        #[cfg(not(feature = "trust-manifest"))]
        line("    Runs in this build require a fingerprint.");
        example("  .jaoignore: scratch/ or seed.dev.sh");
        line("    Hide throwaway directories or internal scripts from discovery.");
    } else {
        println!("jao - discover, inspect, and run workspace scripts");
        println!("  Finds platform scripts recursively and executes them from their own directory.");
        println!();
        println!("USAGE:");
        println!("  jao --list");
        println!("  jao --ci --list");
        println!("  jao --completions <bash|zsh>");
        println!("  jao --fingerprint <SCRIPT_COMMAND>...");
        println!("  jao <SCRIPT_COMMAND>...");
        #[cfg(feature = "trust-manifest")]
        println!("  jao --ci --require-fingerprint <FINGERPRINT> <SCRIPT_COMMAND>...");
        #[cfg(not(feature = "trust-manifest"))]
        println!("  jao --require-fingerprint <FINGERPRINT> <SCRIPT_COMMAND>...");
        println!();
        println!("OPTIONS:");
        option_plain("  -h, --help", "Show this help screen");
        option_plain("      --list", "List runnable scripts as command names plus their resolved file paths");
        option_plain("      --completions <SHELL>", "Print a bash or zsh completion script");
        option_plain(
            "      --fingerprint <SCRIPT_COMMAND>...",
            "Resolve a script command, then print SHA-256 of canonical path + file contents",
        );
        option_plain("      --ci", "Enable CI mode (non-interactive, no config/manifest I/O)");
        option_plain(
            "      --require-fingerprint <FINGERPRINT>",
            "Require exact script fingerprint before running",
        );
        option_plain("  -V, --version", "Print version");
        println!();
        println!("SCRIPT COMMAND INPUT:");
        println!("  Script file stems still use '.' as command separators.");
        println!("  Directories contribute command parts only when they contain a .jaofolder marker.");
        println!("  Discovery also respects .gitignore.");
        println!("  .jaoignore files are applied recursively and exclude matching directories and scripts.");
        println!("  Multi-project example:");
        println!("    apps/frontend/scripts/dev.sh with apps/ and frontend/ marked becomes:");
        println!("    jao apps frontend dev from the workspace root");
        println!("    jao frontend dev from inside apps/");
        println!("    jao dev from inside apps/frontend/");
        println!("  Matching extension is chosen by OS: .sh on Unix-like systems, .bat on Windows.");
        println!("  The script runs with working directory set to the script's folder.");
        println!("  Shell completion can suggest script parts dynamically from the current directory.");
        println!();
        println!("TRUST BEHAVIOR:");
        #[cfg(feature = "trust-manifest")]
        {
            println!("  Running a script requires trust.");
            println!("  Unknown scripts prompt: trust and run? [y/N]");
            println!("  Modified scripts prompt: re-trust and run? [y/N]");
            println!("  In non-interactive mode, unknown/modified scripts fail.");
            println!("  --list prints trust state labels plus command names and resolved paths.");
            println!("  --require-fingerprint can be used in CI mode.");
        }
        #[cfg(not(feature = "trust-manifest"))]
        {
            println!("  This build has trust-manifest disabled.");
            println!("  All runs require --require-fingerprint <FINGERPRINT>.");
            println!("  Runs are always non-interactive and never write trust state.");
            println!("  --list prints command names and resolved paths.");
        }
        println!("  --ci skips config/manifest creation and never prompts.");
        #[cfg(feature = "trust-manifest")]
        println!("  --ci run requires --require-fingerprint <FINGERPRINT>.");
        #[cfg(not(feature = "trust-manifest"))]
        println!("  --ci uses the same fingerprint-required run policy.");
        println!("  --ci --list prints command names and resolved paths (no trust labels).");
        println!();
        println!("EXAMPLES:");
        println!("  jao check");
        println!("    Run check.sh / check.bat if found.");
        println!("  jao test integration");
        println!("    Run test.integration.sh / .bat if found.");
        println!("  jao db reset local");
        println!("    Run db.reset.local.sh / .bat if found.");
        println!("  source <(jao --completions bash)");
        println!("    Install bash completion for the current shell.");
        println!("  jao m<TAB>");
        println!("    Dynamic completion can expand that to myapp, then deeper parts like backend or build.");
        println!("  jao --list");
        #[cfg(feature = "trust-manifest")]
        println!("    Output includes trust state, command name, and resolved path.");
        #[cfg(not(feature = "trust-manifest"))]
        println!("    Output includes command name and resolved path.");
        println!("  jao apps backend build");
        println!("    Example of using .jaofolder in a multi-project repo.");
        println!("  jao --ci --require-fingerprint <FINGERPRINT> db reset local");
        println!("    Run only if the resolved script fingerprint matches exactly.");
        #[cfg(feature = "trust-manifest")]
        println!("  jao --fingerprint db reset local");
        #[cfg(not(feature = "trust-manifest"))]
        println!("  jao --require-fingerprint <FINGERPRINT> db reset local");
        #[cfg(feature = "trust-manifest")]
        println!("    Print the fingerprint you can later require in CI.");
        #[cfg(not(feature = "trust-manifest"))]
        println!("    Runs in this build require a fingerprint.");
        println!("  .jaoignore: scratch/ or seed.dev.sh");
        println!("    Hide throwaway directories or internal scripts from discovery.");
    }

    Ok(())
}

fn should_style() -> bool {
    let no_color = std::env::var_os("NO_COLOR").is_some();
    let force_color = std::env::var("CLICOLOR_FORCE")
        .ok()
        .as_deref()
        == Some("1");
    (std::io::stdout().is_terminal() || force_color) && !no_color
}

fn section(name: &str) {
    println!("{BOLD}{UNDERLINE}{name}:{RESET}");
}

fn option(flag: &str, desc: &str) {
    if flag
        .chars()
        .count()
        <= OPTION_COL_WIDTH
    {
        println!("{BOLD}{flag:<OPTION_COL_WIDTH$}{RESET}{desc}");
    } else {
        println!("{BOLD}{flag}{RESET}");
        println!("{OPTION_DESC_INDENT}{desc}");
    }
}

fn option_plain(flag: &str, desc: &str) {
    if flag
        .chars()
        .count()
        <= OPTION_COL_WIDTH
    {
        println!("{flag:<OPTION_COL_WIDTH$}{desc}");
    } else {
        println!("{flag}");
        println!("{OPTION_DESC_INDENT}{desc}");
    }
}

fn line(text: &str) {
    println!("{text}");
}

fn example(cmd: &str) {
    println!("{GREEN}{cmd}{RESET}");
}
