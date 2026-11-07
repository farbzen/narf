# Usage

```sh
narf [OPTIONS]
```

NARF works on a NixOS configuration flake in a git repository. Each run
updates the flake inputs on request, rebuilds the system with
[nh](https://github.com/nix-community/nh), commits the new generation and
optionally pushes it.

## Options

| Short | Long                    | Value | Effect                                                                  |
| ----- | ----------------------- | ----- | ----------------------------------------------------------------------- |
| `-b`  | `--build-only`          |       | build the new generation without switching                              |
| `-c`  | `--comment`             | text  | add a comment to the resulting commit                                   |
| `-d`  | `--dry`                 |       | only evaluate and state what would be done                              |
| `-f`  | `--flake`               | path  | path to the flake (a git repository); defaults to the saved path        |
| `-H`  | `--host`                | name  | NixOS host configuration to switch to; defaults to the current hostname |
| `-n`  | `--no-rebuild`          |       | skip the rebuild and only push (and squash) existing commits            |
| `-p`  | `--pull`                |       | pull new commits from origin before running                             |
| `-P`  | `--push`                |       | push new commits to origin                                              |
| `-S`  | `--dont-squash-commits` |       | don't squash unpushed commits before pushing (only with `-P`)           |
| `-t`  | `--trace`               |       | show a trace of the rebuild                                             |
| `-u`  | `--update`              |       | update all flake inputs before running                                  |
| `-U`  | `--update-input`        | input | update a specific input before running; repeat for more inputs          |
| `-y`  | `--yes`                 |       | answer all prompts with yes (non-interactive)                           |
| `-h`  | `--help`                |       | print help                                                              |
| `-V`  | `--version`             |       | print version                                                           |

Short flags can be combined, e.g. `narf -upP`.

> [!NOTE]
> `-n` can't be combined with `-u`, `-U`, `-c`, `-b` or `-t`. Uncommitted
> changes are left untouched in this mode.

## Flake path

Without `--flake`, NARF reads the flake path from `~/.config/narf/flake`. When
`--flake` points to a different path than the saved one, NARF offers to save
it for future runs.

## Run

A run consists of these steps:

1. **Pull** (`-p`): `git pull` in the flake repository.
2. **Update** (`-u`, `-U`): `nix flake update` for all or the given inputs.
3. **Stage**: `git add` of all changes in the repository (skipped with `-n`).
4. **Rebuild**: `nh os switch` (or `nh os build` with `-b`) for the host,
   with `--dry` for `-d` and `--show-trace` for `-t`. Without staged changes
   and without `-p`, NARF asks whether to rebuild anyway (`-y` answers yes).
5. **Commit**: if there were staged changes and neither `-d` nor `-b` is set,
   the new generation is committed.
6. **Push** (`-P`, skipped with `-d` and `-b`): a single unpushed commit is
   pushed as it is. Several unpushed commits are squashed into one with the
   message of the latest commit, after a confirmation, unless `-S` is given.

> [!IMPORTANT]
> Pushing needs an upstream branch. Squashing is refused while there are
> staged changes, since they would end up in the squashed commit.

## Commit message

The message describes the current generation from
`nixos-rebuild list-generations`; a comment from `-c` is appended as its last
line:

```text
rebuild(<host>): Generation 42

NixOS 26.11...
Linux 7.2...

Built: 2026-10-22T09:00:00
<comment>
```

## Exit codes

| Code | Meaning           |
| ---- | ----------------- |
| `0`  | success           |
| `1`  | a step failed     |
| `2`  | invalid arguments |

## Examples

```sh
narf --update --push            # update inputs, rebuild, commit and push
narf --dry                      # only show what would be done
narf -U nixpkgs -b              # update only nixpkgs, build without switching
narf -c "new kernel" -P         # rebuild, commit with a comment and push
narf -n -P                      # squash and push existing rebuild commits
narf -f ~/nixos -H laptop -y    # rebuild another host without prompts
```
