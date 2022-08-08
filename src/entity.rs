// Copyright (C) 2022 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! The [`Entity`] type.

use crate::resource::Resource;
use crate::version::Version;

/// A "versionable" entity
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Entity {
    /// The whole entity-describing string
    pub line: String,
    /// The resource part - `reference` without the version
    pub resource: Resource,
    /// The current version
    pub version: Version,
    /// The latest version
    pub latest: Option<Version>,
    /// The updated entity-describing string
    pub updated_line: Option<String>,
}

impl Entity {
    pub fn new(line: String, resource: Resource, version: Version) -> Entity {
        Entity {
            line,
            resource,
            version,
            latest: None,
            updated_line: None,
        }
    }
}
