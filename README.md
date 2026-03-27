<p align="center">
  <img width="316" height="337" alt="Jao-Jao-Jao" src="https://github.com/user-attachments/assets/eb163b40-9283-458c-8d40-1fe94a3183f2" />
</p>
<br />

`jao` finds repo scripts and lets you run them by name.

It is meant for repos that already have shell or batch scripts and want a thin CLI on top, not a bigger task runner.

## Installation

Just run `cargo install jao` ;)

## What It Does

- Finds runnable scripts under the directory you start from
- Matches `.sh` on Unix and `.bat` on Windows
- Lets you run scripts by command parts instead of file paths
- Respects `.gitignore` during discovery
- Supports `.jaofolder` to expose directory names in commands
- Supports recursive `.jaoignore` files to hide scripts or whole areas
- Supports trust prompts for local use and fingerprints for CI

**`jao` runs scripts from the script's own directory, so existing scripts that rely on relative paths keep working.**

## Example 1: Basic Use

Repo:

```text
scripts/
  check.sh
  test.integration.sh
  db.reset.local.sh
```

Commands:

```bash
jao check
jao test integration
jao db reset local
jao --list
```

This is the default model: command parts map to script stems like `check`, `test.integration`, and `db.reset.local`.

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

## Install from source

```bash
cargo install --path .
```

During development:

```bash
cargo run -- --list
```

## Docker

```bash
docker build -t jao .
docker run --rm -it -v "$PWD:/workspace" -w /workspace jao --list
```

## License

Apache-2.0
