use std::io::IsTerminal;

const BOLD: &str = "\x1b[1m";
const UNDERLINE: &str = "\x1b[4m";
const CYAN: &str = "\x1b[36m";
const GREEN: &str = "\x1b[32m";
const RESET: &str = "\x1b[0m";
const OPTION_COL_WIDTH: usize = 28;
const OPTION_DESC_INDENT: &str = "                              ";

pub fn print_help() {
    if should_style() {
        println!("{BOLD}{CYAN}jao - discover, inspect, and run workspace scripts{RESET}");
        line("  Finds platform scripts recursively and executes them from their own directory.");
        println!();
        section("USAGE");
        line("  jao --list");
        line("  jao --ci --list");
        line("  jao --fingerprint <SCRIPT_COMMAND>...");
        line("  jao <SCRIPT_COMMAND>...");
        line("  jao --ci --require-fingerprint <FINGERPRINT> <SCRIPT_COMMAND>...");
        println!();

        section("OPTIONS");
        option("  -h, --help", "Show this help screen");
        option(
            "      --list",
            "List runnable scripts discovered from the current directory downward",
        );
        option(
            "      --fingerprint <SCRIPT_COMMAND>...",
            "Resolve a script command, then print SHA-256 of canonical path + file contents",
        );
        option(
            "      --ci",
            "Enable CI mode (non-interactive, no config/manifest I/O)",
        );
        option(
            "      --require-fingerprint <FINGERPRINT>",
            "In CI mode, require exact script fingerprint before running",
        );
        option("  -V, --version", "Print version");
        println!();

        section("SCRIPT COMMAND INPUT");
        line("  Positional parts are joined with '.' to form the script base name.");
        line("  Example: jao deploy api prod  -> base name deploy.api.prod");
        line("  Matching extension is chosen by OS: .sh on Unix-like systems, .bat on Windows.");
        line("  The script runs with working directory set to the script's folder.");
        println!();

        section("TRUST BEHAVIOR");
        line("  Running a script requires trust.");
        line("  Unknown scripts prompt: trust and run? [y/N]");
        line("  Modified scripts prompt: re-trust and run? [y/N]");
        line("  In non-interactive mode, unknown/modified scripts fail.");
        line("  --list prints trust state labels: trusted, unknown, or modified.");
        line("  --ci skips config/manifest creation and never prompts.");
        line("  --ci run requires --require-fingerprint <FINGERPRINT>.");
        line("  --ci --list prints script paths only (no trust labels).");
        println!();

        section("EXAMPLES");
        example("  jao --fingerprint deploy api prod");
        line("    Resolve deploy.api.prod.sh/.bat, then fingerprint that script file.");
        example("  jao --list");
        line("    Output format: <trust-state> <script-path>.");
        example("  jao --ci --list");
        line("    Output format: <script-path>.");
        example("  jao --ci --require-fingerprint <FINGERPRINT> test integrations myapp");
        line("    Run only if resolved script fingerprint matches exactly.");
        example("  jao test");
        line("    Run test.sh / test.bat if found.");
        example("  jao deploy api prod");
        line("    Run deploy.api.prod.sh / .bat if found.");
    } else {
        println!("jao - discover, inspect, and run workspace scripts");
        println!(
            "  Finds platform scripts recursively and executes them from their own directory."
        );
        println!();
        println!("USAGE:");
        println!("  jao --list");
        println!("  jao --ci --list");
        println!("  jao --fingerprint <SCRIPT_COMMAND>...");
        println!("  jao <SCRIPT_COMMAND>...");
        println!("  jao --ci --require-fingerprint <FINGERPRINT> <SCRIPT_COMMAND>...");
        println!();
        println!("OPTIONS:");
        option_plain("  -h, --help", "Show this help screen");
        option_plain(
            "      --list",
            "List runnable scripts discovered from the current directory downward",
        );
        option_plain(
            "      --fingerprint <SCRIPT_COMMAND>...",
            "Resolve a script command, then print SHA-256 of canonical path + file contents",
        );
        option_plain(
            "      --ci",
            "Enable CI mode (non-interactive, no config/manifest I/O)",
        );
        option_plain(
            "      --require-fingerprint <FINGERPRINT>",
            "In CI mode, require exact script fingerprint before running",
        );
        option_plain("  -V, --version", "Print version");
        println!();
        println!("SCRIPT COMMAND INPUT:");
        println!("  Positional parts are joined with '.' to form the script base name.");
        println!("  Example: jao deploy api prod  -> base name deploy.api.prod");
        println!(
            "  Matching extension is chosen by OS: .sh on Unix-like systems, .bat on Windows."
        );
        println!("  The script runs with working directory set to the script's folder.");
        println!();
        println!("TRUST BEHAVIOR:");
        println!("  Running a script requires trust.");
        println!("  Unknown scripts prompt: trust and run? [y/N]");
        println!("  Modified scripts prompt: re-trust and run? [y/N]");
        println!("  In non-interactive mode, unknown/modified scripts fail.");
        println!("  --list prints trust state labels: trusted, unknown, or modified.");
        println!("  --ci skips config/manifest creation and never prompts.");
        println!("  --ci run requires --require-fingerprint <FINGERPRINT>.");
        println!("  --ci --list prints script paths only (no trust labels).");
        println!();
        println!("EXAMPLES:");
        println!("  jao --fingerprint deploy api prod");
        println!("    Resolve deploy.api.prod.sh/.bat, then fingerprint that script file.");
        println!("  jao --list");
        println!("    Output format: <trust-state> <script-path>.");
        println!("  jao --ci --list");
        println!("    Output format: <script-path>.");
        println!("  jao --ci --require-fingerprint <FINGERPRINT> test integrations myapp");
        println!("    Run only if resolved script fingerprint matches exactly.");
        println!("  jao test");
        println!("    Run test.sh / test.bat if found.");
        println!("  jao deploy api prod");
        println!("    Run deploy.api.prod.sh / .bat if found.");
    }
}

fn should_style() -> bool {
    let no_color = std::env::var_os("NO_COLOR").is_some();
    let force_color = std::env::var("CLICOLOR_FORCE").ok().as_deref() == Some("1");
    (std::io::stdout().is_terminal() || force_color) && !no_color
}

fn section(name: &str) {
    println!("{BOLD}{UNDERLINE}{name}:{RESET}");
}

fn option(flag: &str, desc: &str) {
    if flag.chars().count() <= OPTION_COL_WIDTH {
        println!("{BOLD}{flag:<OPTION_COL_WIDTH$}{RESET}{desc}");
    } else {
        println!("{BOLD}{flag}{RESET}");
        println!("{OPTION_DESC_INDENT}{desc}");
    }
}

fn option_plain(flag: &str, desc: &str) {
    if flag.chars().count() <= OPTION_COL_WIDTH {
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
