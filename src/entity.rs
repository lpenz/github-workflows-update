// Copyright (C) 2022 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! The [`Entity`] type.

use regex::Regex;
use std::fmt;

use crate::error::Error;
use crate::error::Result;
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

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Resource {
    Docker { container: String },
    GhAction { user: String, repo: String },
}

impl Resource {
    pub fn new_docker(container: String) -> Resource {
        Resource::Docker { container }
    }

    pub fn new_ghaction(user: String, repo: String) -> Resource {
        Resource::GhAction { user, repo }
    }

    pub fn parse(input: &str) -> Result<(Self, Version), Error> {
        let re_docker = Regex::new(r"^docker://(?P<resource>[^:]+):(?P<version>[^:]+)$").unwrap();
        if let Some(m) = re_docker.captures(input) {
            let version_str = m.name("version").unwrap().as_str();
            let version = Version::new(version_str)
                .ok_or_else(|| Error::VersionParsing(version_str.into()))?;
            return Ok((
                Resource::new_docker(m.name("resource").unwrap().as_str().into()),
                version,
            ));
        }
        let re_github =
            Regex::new(r"^(?P<user>[^/]+)/(?P<repo>[^@]+)@(?P<version>[^@]+)$").unwrap();
        if let Some(m) = re_github.captures(input) {
            let version_str = m.name("version").unwrap().as_str();
            let version = Version::new(version_str)
                .ok_or_else(|| Error::VersionParsing(version_str.into()))?;
            return Ok((
                Resource::new_ghaction(
                    m.name("user").unwrap().as_str().into(),
                    m.name("repo").unwrap().as_str().into(),
                ),
                version,
            ));
        }
        Err(Error::ResourceParseError(input.into()))
    }
}

impl fmt::Display for Resource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Resource::Docker { container } => format!("docker://{}", container),
                Resource::GhAction { user, repo } => format!("github://{}/{}", user, repo),
            }
        )
    }
}
