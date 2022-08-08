// Copyright (C) 2022 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! The [`Resource`] type.

use regex::Regex;
use std::fmt;
use tracing::instrument;

use crate::error::Error;
use crate::error::Result;
use crate::updater;
use crate::version::Version;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum Resource {
    Docker { container: String },
    GhAction { user: String, repo: String },
}

impl Resource {
    #[instrument(level = "debug")]
    pub fn new_docker(container: String) -> Resource {
        Resource::Docker { container }
    }

    #[instrument(level = "debug")]
    pub fn new_ghaction(user: String, repo: String) -> Resource {
        Resource::GhAction { user, repo }
    }

    pub fn is_docker(&self) -> bool {
        match self {
            Resource::Docker { .. } => true,
            Resource::GhAction { .. } => false,
        }
    }

    pub fn is_github(&self) -> bool {
        match self {
            Resource::Docker { .. } => false,
            Resource::GhAction { .. } => true,
        }
    }

    #[instrument(level = "debug")]
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

    #[instrument(level = "debug")]
    pub fn url(&self) -> String {
        match self {
            Resource::Docker { container } => format!(
                "https://registry.hub.docker.com/v1/repositories/{}/tags",
                container
            ),
            Resource::GhAction { user, repo } => format!(
                "https://api.github.com/repos/{}/{}/git/matching-refs/tags",
                user, repo
            ),
        }
    }

    #[instrument(level = "debug")]
    pub fn versioned_string(&self, version: &Version) -> String {
        match self {
            Resource::Docker { .. } => format!("{}:{}", self, version),
            Resource::GhAction { .. } => format!("{}@{}", self, version),
        }
    }

    #[instrument(level = "debug")]
    pub async fn get_versions(&self) -> Result<Vec<Version>> {
        match self {
            Resource::Docker { .. } => updater::docker::get_versions(&self.url()).await,
            Resource::GhAction { .. } => updater::github::get_versions(&self.url()).await,
        }
    }
}

impl fmt::Display for Resource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Resource::Docker { container } => format!("docker://{}", container),
                Resource::GhAction { user, repo } => format!("{}/{}", user, repo),
            }
        )
    }
}
