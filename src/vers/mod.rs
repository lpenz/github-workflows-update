// Copyright (C) 2022 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use anyhow::anyhow;
use anyhow::Result;
use versions::Version;

pub mod docker;

pub mod resolver;

pub async fn discover_latest_version(
    resolver: resolver::Client,
    resource: &str,
) -> Result<Version> {
    if let Some(path) = resource.strip_prefix("docker://") {
        docker::docker_latest_version(resolver, path).await
    } else {
        Err(anyhow!("error parsing resource type for {:?}", resource))
    }
}
