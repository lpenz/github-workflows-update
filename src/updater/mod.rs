// Copyright (C) 2022 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! The resolver can deal with different upstream API's by using the
//! [`Upd`] trait, which is generic over the currently supported
//! [`docker`] and [`github`] upstreams.

use async_trait::async_trait;

use crate::entity::Entity;
use crate::error::Error;
use crate::error::Result;
use crate::resource::Resource;
use crate::version::Version;

#[async_trait]
pub trait Updater: std::fmt::Debug {
    async fn get_versions(&self, url: &str) -> Result<Vec<Version>>;
    fn updated_line(&self, entity: &Entity) -> Option<String>;
}

pub mod docker;
pub mod github;

#[derive(Debug)]
pub enum Upd {
    Docker(docker::Docker),
    Github(github::Github),
}

#[async_trait]
impl Updater for Upd {
    async fn get_versions(&self, url: &str) -> Result<Vec<Version>> {
        match self {
            Upd::Docker(i) => i.get_versions(url),
            Upd::Github(i) => i.get_versions(url),
        }
        .await
    }
    fn updated_line(&self, entity: &Entity) -> Option<String> {
        match self {
            Upd::Docker(i) => i.updated_line(entity),
            Upd::Github(i) => i.updated_line(entity),
        }
    }
}

pub fn updater_for(resource: &Resource) -> Result<impl Updater> {
    if resource.is_docker() {
        Ok(Upd::Docker(docker::Docker::default()))
    } else if resource.is_github() {
        Ok(Upd::Github(github::Github::default()))
    } else {
        Err(Error::UpdaterNotFound(resource.to_string()))
    }
}
