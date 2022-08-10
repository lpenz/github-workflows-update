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
//!   - [`workflow`]: workflow file parsing, into [`workflow::Workflow`] type.
//!     The workflow has the the set of resource-versions that the workflow
//!     `uses`, and also fetches all latest versions using the proxy.
//!   - [`proxy`]: a proxy [`proxy::Server`] that makes async
//!     requests and caches the results.

pub mod cmd;
pub mod error;
pub mod processor;
pub mod proxy;
pub mod resource;
pub mod updater;
pub mod version;
pub mod workflow;
