// Copyright (C) 2022 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use versions::Version;

/// A "versionable" entity
#[derive(Debug, Default)]
pub struct Entity {
    /// Workflow job
    pub job: String,
    /// The whole entity-describing string
    pub reference: String,
    /// The resource part - `reference` without the version
    pub resource: String,
    /// The version
    pub version: Version,
}
