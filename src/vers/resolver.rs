// Copyright (C) 2022 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use anyhow::anyhow;
use anyhow::Result;
use versions::Version;

use crate::vers::docker;

#[derive(Debug)]
pub struct Server {}

#[derive(Debug)]
pub struct Client {}

impl Server {
    pub fn new() -> Server {
        Server {}
    }

    pub fn new_client(&self) -> Client {
        Client {}
    }
}

impl Default for Server {
    fn default() -> Self {
        Self::new()
    }
}

impl Client {
    pub async fn get_latest_version(&self, resource: &str) -> Result<Version> {
        if let Some(url) = docker::url(resource) {
            docker::get_latest_version(&url).await
        } else {
            Err(anyhow!("could not parse resource"))
        }
    }
}
