# jao

`jao` is a small CLI for people who already have a pile of repo scripts and do not want to invent a bigger tool just to run them.

It walks your workspace, finds runnable scripts for your platform, and lets you call them by a simple command name instead of remembering where they live.

If you have ever had a repo full of things like:

- `scripts/check.sh`
- `ops/deploy.api.prod.sh`
- `tools/test.integration.sh`

and thought "I just want to run the thing, not maintain another task runner", that is the job `jao` is trying to do.

## What It Gives You

- Recursive script discovery from the directory you run it in
- A simple command style: `jao deploy api prod`
- Optional `.jaofolder` markers to expose directory names as command prefixes
- Cross-platform matching: `.sh` on Unix, `.bat` on Windows
- Script fingerprinting for CI or locked-down runs
- Trust tracking for local interactive use

The default build keeps a trust manifest so local runs are practical:

- first run of a script asks whether you trust it
- changed scripts ask again
- CI can require an exact fingerprint and skip all prompts

## How It Works

`jao` joins positional arguments with dots and looks for a matching script file.
Script file stems still define the tail of the command, while directories only
show up in the command when they contain a `.jaofolder` marker file.

Examples:

```bash
jao test
jao check all
jao deploy api prod
```

Those resolve to script base names like:

- `test`
- `check.all`
- `deploy.api.prod`

Then `jao` finds the matching `.sh` or `.bat` file somewhere under the current directory and runs it from that script's own folder.

That last part matters. A lot of repo scripts assume their working directory is the folder they live in. `jao` respects that.

### Folder Markers

If you have a script at `myapp/backend/scripts/build.sh` and both `myapp/` and
`backend/` contain a `.jaofolder` file, then the command changes with where you
run `jao`:

- from outside `myapp/`: `jao myapp backend build`
- from inside `myapp/`: `jao backend build`
- from inside `myapp/backend/`: `jao build`

Directories without `.jaofolder` stay invisible, so folders like `scripts/`
can keep organizing files without polluting the command name.

## Common Commands

List everything `jao` can run from the current repo:

```bash
jao --list
```

This prints logical command names together with the script path they resolve to.

Print a script fingerprint:

```bash
jao --fingerprint deploy api prod
```

Run a script normally:

```bash
jao deploy api prod
```

Run in CI with a required fingerprint:

```bash
jao --ci --require-fingerprint <FINGERPRINT> deploy api prod
```

## Why Use This Instead of Just Calling Scripts Directly?

Because direct script paths rot fast.

- People move files around
- Different teams stash scripts in different folders
- Nobody remembers exact relative paths
- CI wants something stricter than "just trust me"

`jao` gives you one stable way to discover, inspect, and run scripts without forcing your repo into a full task-runner ecosystem.

It is intentionally boring software. That is the point.

## Trust Model

By default, `jao` stores config under `~/.jao/` and keeps a trust manifest there.

Interactive runs:

- unknown scripts prompt before running
- modified scripts prompt again

Non-interactive runs:

- `--ci` never prompts
- CI runs require `--require-fingerprint`

This makes local usage convenient without turning CI into a guess.

## Install

Build it locally with Cargo:

```bash
cargo install --path .
```

Or run it directly during development:

```bash
cargo run -- --list
```

## Docker

Build the image:

```bash
docker build -t jao .
```

Run it against your current workspace:

```bash
docker run --rm -it -v "$PWD:/workspace" -w /workspace jao --list
```

## License

`jao` is licensed under Apache-2.0.

If you redistribute it or ship a derivative, keep the license and notice text in place. If you improve it, upstream PRs are preferred.

## Good Fit

`jao` is a good fit if:

- your repo already uses shell or batch scripts
- you want a thin CLI on top of them
- you want trust/fingerprint checks without much ceremony

It is probably not the right tool if you want a full workflow engine, dependency graph, task cache, or language-specific build system. It is just a clean way to find and run scripts.
