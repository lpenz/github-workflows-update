// Copyright (C) 2022 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use anyhow::anyhow;
use anyhow::Result;
use versions::Version;

pub mod docker;

pub async fn discover_latest_version(resource: &str) -> Result<Version> {
    if let Some(path) = resource.strip_prefix("docker://") {
        docker::docker_latest_version(path).await
    } else {
        Err(anyhow!("error parsing resource type for {:?}", resource))
    }
}
