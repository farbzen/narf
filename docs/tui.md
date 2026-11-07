# TUI

```sh
narf-tui
```

`narf-tui` is a terminal frontend for NARF: it lets you pick the options in a
form, shows the state of the flake repository and runs NARF in an embedded
terminal, so its prompts keep working.

## Layout

- **Options**: every NARF option with its flag letter; toggles show `[x]` or
  `[ ]`, text options their value.
- **Update inputs**: the root inputs from the flake's `flake.lock`; selected
  inputs are passed as `-U <input>`. The list is ignored while "Update all
  inputs" (`-u`) is set.
- **Status**: flake path, host, upstream branch, unpushed commits, `HEAD` and
  the uncommitted changes that NARF will stage and commit.
- **Hints**: combinations NARF accepts but ignores, such as `-S` without `-P`,
  and a host that is not in the flake's `nixosConfigurations`.
- **Command**: the resulting `narf` command line.

The flake is resolved like NARF does: the value of the "Flake" option, or the
path saved in `~/.config/narf/flake`.

## Keys

| Key                     | Action                                        |
| ----------------------- | --------------------------------------------- |
| `j`/`k`, `↓`/`↑`        | move                                          |
| `g`/`G`, `Home`/`End`   | first/last entry                              |
| `Space`, `Enter`        | toggle an option or input, edit a text option |
| flag letter (`p` … `H`) | select the option and toggle or edit it       |
| `Tab`, `h`/`l`          | switch between options and inputs             |
| `r`                     | run NARF                                      |
| `R`                     | reload the status                             |
| `q`, `Esc`, `Ctrl+C`    | quit                                          |

While editing a text option:

| Key      | Action                       |
| -------- | ---------------------------- |
| `Enter`  | apply                        |
| `Esc`    | cancel and restore the value |
| `Ctrl+U` | clear                        |
| `Ctrl+⌫` | delete the last word         |

## Host picker

Activating "Host" (`H`) opens a list of the flake's `nixosConfigurations`,
evaluated in the background. The first entry keeps the default (the
hostname), the last one, "custom…", lets you type a host by hand.

## Running NARF

`r` starts NARF with the selected options in an embedded terminal. Keys are
passed to NARF while it runs; `PageUp`/`PageDown` scroll the output. When
NARF has finished, `q`, `Esc` or `Enter` returns to the form and reloads the
status.
