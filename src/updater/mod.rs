// Copyright (C) 2022 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use anyhow::anyhow;
use anyhow::Result;
use async_trait::async_trait;
use versions::Version;

use crate::entity::Entity;

#[async_trait]
pub trait Updater: std::fmt::Debug {
    fn url(&self, resource: &str) -> Option<String>;
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
    fn url(&self, resource: &str) -> Option<String> {
        match self {
            Upd::Docker(i) => i.url(resource),
            Upd::Github(i) => i.url(resource),
        }
    }
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

pub fn updater_for(resource: &str) -> Result<impl Updater> {
    if let Some(_url) = docker::url(resource) {
        Ok(Upd::Docker(docker::Docker::default()))
    } else if let Some(_url) = github::url(resource) {
        Ok(Upd::Github(github::Github::default()))
    } else {
        Err(anyhow!("no updater found"))
    }
}
