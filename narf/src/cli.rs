//! Command line arguments.

use clap::Parser;

/// NARF (NixOS Advanced Rebuild Flake) - Rebuilding nix flakes and creating commits
#[derive(Parser, Debug)]
#[command(name = "narf", version)]
pub struct Args {
    /// Build the new generation without switching
    #[arg(short, long)]
    pub build_only: bool,

    /// Add a comment to the resulting commit
    #[arg(short, long)]
    pub comment: Option<String>,

    /// Run in 'dry' mode. Only evaluates and states what WOULD be done. Does not take any effective action
    #[arg(short, long)]
    pub dry: bool,

    /// Path to the flake (must be a git repository). Checks configuration in ".config/narf/flake" by default
    #[arg(short, long)]
    pub flake: Option<String>,

    /// NixOS host configuration to switch to. Uses current hostname by default
    #[arg(short = 'H', long)]
    pub host: Option<String>,

    /// Skip the rebuild and only push (and squash) existing commits. Uncommitted changes are left untouched
    #[arg(
        short = 'n',
        long,
        conflicts_with_all = ["update", "update_input", "comment", "build_only", "trace"]
    )]
    pub no_rebuild: bool,

    /// Pull new commits from origin before running
    #[arg(short, long)]
    pub pull: bool,

    /// Push new commits to origin
    #[arg(short = 'P', long)]
    pub push: bool,

    /// Don't squash commits before pushing to origin (only applicable when combined with -P/--push)
    #[arg(short = 'S', long)]
    pub dont_squash_commits: bool,

    /// Show trace of the rebuild (useful if the rebuild keeps failing)
    #[arg(short, long)]
    pub trace: bool,

    /// Update all inputs before running
    #[arg(short, long)]
    pub update: bool,

    /// Update a specific input before running. Can be used multiple times to specify more than one specific input
    #[arg(short = 'U', long)]
    pub update_input: Vec<String>,

    /// Say yes to all (Non-interactive)
    #[arg(short, long)]
    pub yes: bool,
}

#[cfg(test)]
mod tests {
    use clap::{CommandFactory, Parser};

    use super::Args;

    fn parse(args: &[&str]) -> Args {
        Args::try_parse_from(std::iter::once("narf").chain(args.iter().copied()))
            .expect("valid arguments")
    }

    #[test]
    fn command_definition_is_valid() {
        Args::command().debug_assert();
    }

    #[test]
    fn everything_is_off_without_arguments() {
        let args = parse(&[]);
        assert!(!args.build_only && !args.dry && !args.no_rebuild);
        assert!(!args.pull && !args.push && !args.dont_squash_commits);
        assert!(!args.trace && !args.update && !args.yes);
        assert!(args.comment.is_none() && args.flake.is_none() && args.host.is_none());
        assert!(args.update_input.is_empty());
    }

    #[test]
    fn combined_short_flags_set_every_switch() {
        let args = parse(&["-bdptuyPS"]);
        assert!(args.build_only && args.dry);
        assert!(args.pull && args.push && args.dont_squash_commits);
        assert!(args.trace && args.update && args.yes);
    }

    #[test]
    fn no_rebuild_combines_with_pull_and_push() {
        let args = parse(&["-npPS"]);
        assert!(args.no_rebuild && args.pull && args.push && args.dont_squash_commits);
    }

    #[test]
    fn no_rebuild_rejects_rebuild_options() {
        for conflicting in [
            &["-u"][..],
            &["-U", "nixpkgs"],
            &["-c", "comment"],
            &["-b"],
            &["-t"],
        ] {
            let args = ["narf", "-n"]
                .into_iter()
                .chain(conflicting.iter().copied());
            assert!(Args::try_parse_from(args).is_err(), "{conflicting:?}");
        }
    }

    #[test]
    fn values_are_read_from_short_and_long_options() {
        let short = parse(&["-c", "new kernel", "-f", "/etc/nixos", "-H", "laptop"]);
        let long = parse(&[
            "--comment",
            "new kernel",
            "--flake",
            "/etc/nixos",
            "--host",
            "laptop",
        ]);
        for args in [short, long] {
            assert_eq!(args.comment.as_deref(), Some("new kernel"));
            assert_eq!(args.flake.as_deref(), Some("/etc/nixos"));
            assert_eq!(args.host.as_deref(), Some("laptop"));
        }
    }

    #[test]
    fn update_input_can_be_repeated() {
        let args = parse(&["-U", "nixpkgs", "--update-input", "home-manager"]);
        assert_eq!(args.update_input, ["nixpkgs", "home-manager"]);
    }

    #[test]
    fn unknown_flags_are_rejected() {
        assert!(Args::try_parse_from(["narf", "--unknown"]).is_err());
    }

    #[test]
    fn options_require_a_value() {
        assert!(Args::try_parse_from(["narf", "--host"]).is_err());
    }
}
