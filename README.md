<p align="center">
  <img width="316" height="337" alt="Jao-Jao-Jao" src="https://github.com/user-attachments/assets/eb163b40-9283-458c-8d40-1fe94a3183f2" />
</p>
<p align="center">
  <a href="https://github.com/RoyPrinsGH/jao/actions/workflows/dev.yml">
    <img alt="Dev CI" src="https://github.com/RoyPrinsGH/jao/actions/workflows/dev.yml/badge.svg?branch=master" />
  </a>
  <a href="https://github.com/RoyPrinsGH/jao/actions/workflows/release.yml">
    <img alt="Release CI" src="https://github.com/RoyPrinsGH/jao/actions/workflows/release.yml/badge.svg?branch=release" />
  </a>
</p>
<br />

`jao` (from the Hindi "जाओ" meaning "Go!" :D) runs scripts as if you made your own, repo specific CLI

Compared with tools like `make`, `just`, or `npm run`, `jao` is a better fit when:

- you don't want to commit to a full task runner, and want something both seamlessly added and seamlessly removed from your workflow
- the scripts already exist and you do not want to rewrite them
- the same script names appear in multiple subprojects
- you want commands to get shorter as you move deeper into the repo (as in, jao is aware of your current working directory)
- you want changes in scripts to enforce detectable changes in anything depending on them, with the fingerprinting system

## Quick Start

Repo:

```text
scripts/
  check.sh
  test.integration.sh
  db.reset.local.sh
```

Commands:

```bash
jao --list
jao check
jao test integration
jao db reset local
```

This is the default model: command parts map to script stems like `check`, `test.integration`, and `db.reset.local`.

## What It Does

- Finds runnable scripts under the directory you start from
- Matches `.sh` on Unix and `.bat` on Windows
- Lets you run scripts by command parts instead of file paths
- Respects `.gitignore` during discovery
- Supports `.jaofolder` to expose directory names in commands
- Supports recursive `.jaoignore` files to hide scripts or whole areas
- Supports trust prompts for local use and fingerprints for CI
- Prints shell completion scripts for Bash and Zsh

**`jao` runs scripts from the script's own directory, so existing scripts that rely on relative paths keep working.**

## Shell Completion

Bash:

```bash
source <(jao --completions bash)
```

Zsh:

```zsh
source <(jao --completions zsh)
```

Then `jao` can complete discovered script parts from the current directory:

```text
jao m<TAB>         -> myapp
jao myapp <TAB>    -> backend frontend
jao myapp backend b<TAB> -> build
```

Obviously you can add this to e.g. your `.bashrc` to enable completions always.

## Example 2: `.jaofolder` In A Multi-Project Repo

Repo:

```text
apps/
  .jaofolder
  frontend/
    .jaofolder
    scripts/
      build.sh
      dev.sh
  backend/
    .jaofolder
    scripts/
      build.sh
      dev.sh
```

From the repo root:

```bash
jao apps frontend dev
jao apps backend build
```

From inside `apps/`:

```bash
jao frontend dev
jao backend build
```

From inside `apps/backend/`:

```bash
jao dev
jao build
```

Use this when multiple projects have the same script names and you want commands to get shorter as you move deeper into the repo.

## Example 3: `.jaoignore` For Internal Or Throwaway Scripts

Repo:

```text
.jaoignore
scripts/
  check.sh
scratch/
  scripts/
    one-off-fix.sh
services/
  api/
    .jaofolder
    .jaoignore
    scripts/
      migrate.dev.sh
      seed.dev.sh
```

Root `.jaoignore`:

```text
scratch/
```

`services/api/.jaoignore`:

```text
seed.dev.sh
```

Result:

- `scratch/` is not walked at all
- `seed.dev.sh` is hidden from `jao --list`
- `migrate.dev.sh` still resolves normally

Use this when a repo contains one-off maintenance scripts, experiments, or internal scripts that should not be part of the public command surface.

## Example 4: Fingerprinting In CI

Print the fingerprint for a script:

```bash
jao --fingerprint db reset local
```

Use that value in CI:

```bash
jao --ci --require-fingerprint <FINGERPRINT> db reset local
```

That gives CI an exact content check instead of trusting whatever script happens to be present.

For local interactive use, `jao` keeps a trust manifest under `~/.jao/`:

- unknown scripts ask before running
- modified scripts ask again

## Installation

### Prebuilt binaries

Download the latest archive for Linux, macOS, or Windows from [GitHub Releases](https://github.com/RoyPrinsGH/jao/releases).

Each release from v0.3.7+ includes platform-specific archives plus a `SHA256SUMS` file.

On Linux and macOS, place the binary on your `PATH` and mark it executable if needed:

```bash
tar -xzf jao-*.tar.gz
chmod +x ./jao
mv ./jao /usr/local/bin/jao
```

On Windows, extract the `.zip` archive and place `jao.exe` somewhere on your `PATH`.

### crates.io

```bash
cargo install jao
```

### Docker

```bash
docker pull royprinsgh/jao:latest
```

## License

Apache-2.0
