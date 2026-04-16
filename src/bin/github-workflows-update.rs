// Copyright (C) 2022 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use color_eyre::Result;

pub fn main() -> Result<()> {
    color_eyre::install()?;
    github_workflows_update::cmd::main()
}
