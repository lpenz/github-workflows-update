// Copyright (C) 2022 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use clap::Parser;
use clap::ValueEnum;

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "Check github workflows for actions that can be updated",
    long_about = "github-workflows-update reads all github workflow and checks the latest
available versions of all github actions and workflow dispatches used, showing
which ones can be updated and optionally updating them automatically."
)]
pub struct Cli {
    /// Don't update the workflows, just print what would be done
    #[clap(short = 'n', long = "dry-run")]
    pub dryrun: bool,
    /// Output format for the outdated action messages
    #[clap(short = 'f', long, value_enum, value_parser)]
    pub output_format: Option<OutputFormat>,
    /// Return error if any outdated actions are found
    #[clap(long)]
    pub error_on_outdated: bool,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Default, Debug)]
pub enum OutputFormat {
    #[default]
    Standard,
    /// Generate messages as github action warnings
    GithubWarning,
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn test_cli() {
        Cli::command().debug_assert();
    }

    #[test]
    fn test_parse_args() {
        let args = Cli::parse_from(&["test", "-n", "-f", "github-warning", "--error-on-outdated"]);
        assert!(args.dryrun);
        assert_eq!(args.output_format, Some(OutputFormat::GithubWarning));
        assert!(args.error_on_outdated);
    }
}
