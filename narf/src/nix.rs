//! Calls to nix, nh and nixos-rebuild.

use std::{
    path::Path,
    process::{Command, Stdio},
};

use crate::{cli::Args, flake::Flake, succeeds};

pub fn rebuild(flake: &Flake, args: &Args) -> bool {
    let mut cmd = Command::new("nh");
    cmd.args(["os", flake.action]).arg(&flake.path).args([
        "-H",
        &flake.host,
        "--accept-flake-config",
    ]);

    if args.dry {
        cmd.arg("--dry");
    }
    if args.trace {
        cmd.arg("--show-trace");
    }

    succeeds(&mut cmd)
}

/// Updates the given inputs, or all of them if `inputs` is empty.
pub fn update_inputs(path: &Path, inputs: &[String]) -> bool {
    succeeds(
        Command::new("nix")
            .args(["flake", "update", "--flake"])
            .arg(path)
            .args(inputs),
    )
}

/// Describes the current generation, e.g.
/// `rebuild(host): Generation 42\n\nNixOS 26.11...\nLinux 7.2...\n\nBuilt: 2026-10-22T09:00:00`.
pub fn commit_message(host: &str) -> String {
    let Ok(output) = Command::new("nixos-rebuild")
        .arg("list-generations")
        .stderr(Stdio::null())
        .output()
    else {
        return String::new();
    };

    describe_current_generation(host, &String::from_utf8_lossy(&output.stdout))
}

fn describe_current_generation(host: &str, generations: &str) -> String {
    generations
        .lines()
        .map(|line| line.split_whitespace().collect::<Vec<_>>())
        .find(|fields| fields.len() == 8 && fields[7] == "True")
        .map(|f| {
            format!(
                "rebuild({host}): Generation {}\n\nNixOS {}\nLinux {}\n\nBuilt: {}T{}",
                f[0], f[3], f[4], f[1], f[2]
            )
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::describe_current_generation;

    const GENERATIONS: &str = "\
Generation  Build-date           NixOS version           Kernel       Configuration Revision  Specialisation  Current
738         2026-11-02 12:58:09  26.11.20261020.02c087f  7.2.11-zen1  Unknown                 []              True
737         2026-11-02 12:16:45  26.11.20261020.02c087f  7.2.11-zen1  Unknown                 []              False
";

    #[test]
    fn describes_the_current_generation() {
        assert_eq!(
            describe_current_generation("laptop", GENERATIONS),
            "rebuild(laptop): Generation 738\n\n\
             NixOS 26.11.20261020.02c087f\n\
             Linux 7.2.11-zen1\n\n\
             Built: 2026-11-02T12:58:09"
        );
    }

    #[test]
    fn ignores_generations_that_are_not_current() {
        let without_current = GENERATIONS.replace("True", "False");
        assert_eq!(describe_current_generation("laptop", &without_current), "");
    }

    #[test]
    fn is_empty_without_generations() {
        assert_eq!(describe_current_generation("laptop", ""), "");
    }
}
