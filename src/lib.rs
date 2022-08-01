// Copyright (C) 2022 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

/// Check if github actions used in a workflow can be updated
///
/// # Code structure
///
/// - cmd: command line arguments
/// - error: error type
/// - version: type wrapper for versions; currently using [semver::version] with [lenient_version]
///
///
pub mod cmd;
pub mod entity;
pub mod error;
pub mod processor;
pub mod resolver;
pub mod updater;
pub mod version;
pub mod workflow;
