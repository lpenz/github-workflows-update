// Copyright (C) 2022 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! Check if github actions used in a workflow can be updated
//!
//! # Code structure
//!
//! - Utility modules:
//!   - [`cmd`]: command line arguments parsing and main function.
//!   - [`error`]: `Error` and `Result` types.
//!   - [`version`]: type wrapper for versions; currently using.
//!     [`semver`] with [`lenient_semver`]
//! - Main functionality:
//!   - [`processor`]: top level file processing function.
//!   - [`workflow`]: workflow file parsing, into [`workflow::Workflow`] type. A
//!     workflow can have one or more [`entity::Entity`]s that represent
//!     resource with a version.
//!   - [`entity`]: the [`entity::Entity`] type.
//!   - [`resolver`]: a resolver [`resolver::Server`] that makes async
//!     requests and caches the result, and an async
//!     [`resolver::Client`] that fills the `entity.latest` field with
//!     the latest version of the upstream entity.
//!   - [`updater`]: the resolver can deal with different upstream
//!     API's by using the [`updater::Upd`] trait, which is generic
//!     over the currently supported [`updater::docker`] and
//!     [`updater::github`] upstreams.

pub mod cmd;
pub mod entity;
pub mod error;
pub mod processor;
pub mod resolver;
pub mod updater;
pub mod version;
pub mod workflow;
