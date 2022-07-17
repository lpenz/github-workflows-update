// Copyright (C) 2022 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

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
