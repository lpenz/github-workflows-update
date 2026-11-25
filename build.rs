// Copyright (C) 2020 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use clap::CommandFactory;
use clap_complete::generate_to;
use clap_complete::shells::Bash;
use clap_complete::shells::Fish;
use clap_complete::shells::Zsh;
use color_eyre::{Result, eyre::Context, eyre::eyre};
use std::env;
use std::fs::{self, File};
use std::io::Write;
use std::path;

include!("src/cli.rs");

fn generate_man_page<P: AsRef<path::Path>>(outdir: P) -> Result<()> {
    let outdir = outdir.as_ref();
    let man_path = outdir.join("github-workflows-update.1");
    let cmd = Cli::command();
    let manual: man::Manual = clap2man::Manual::try_from(&cmd)
        .wrap_err("error converting clap command to manual")?
        .into();
    let manpage = manual
        .example(
            man::prelude::Example::new()
                .text(
                    "Update all actions used in all github workflows \
                       under the current repository",
                )
                .command("github-workflows-update"),
        )
        .example(
            man::prelude::Example::new()
                .text("Show outdated actions without updating them")
                .command("github-workflows-update -n"),
        )
        .render();
    File::create(man_path)?.write_all(manpage.as_bytes())?;
    Ok(())
}

/// Return the profile directory (e.g. `target/release`) that contains OUT_DIR.
///
/// Cargo no longer adds a hash suffix to the build script directory, so its
/// depth is not stable. Find it by walking up from OUT_DIR instead of popping
/// a fixed number of components.
fn profile_dir(outdir: &path::Path, profile: &str) -> Result<path::PathBuf> {
    outdir
        .ancestors()
        .find(|dir| dir.file_name().is_some_and(|name| name == profile))
        .map(|dir| dir.to_path_buf())
        .ok_or_else(|| eyre!("could not find {profile:?} directory in OUT_DIR {outdir:?}"))
}

fn main() -> Result<()> {
    let outdir =
        path::PathBuf::from(env::var_os("OUT_DIR").ok_or_else(|| eyre!("error getting OUT_DIR"))?);
    // Generate assets next to the binary, where the packaging metadata expects them.
    let outdir = profile_dir(&outdir, &env::var("PROFILE")?)?;

    fs::create_dir_all(&outdir)?;
    generate_man_page(&outdir)?;

    let mut cmd = Cli::command();
    generate_to(Bash, &mut cmd, "github-workflows-update", &outdir)?;
    generate_to(Fish, &mut cmd, "github-workflows-update", &outdir)?;
    generate_to(Zsh, &mut cmd, "github-workflows-update", &outdir)?;

    Ok(())
}
