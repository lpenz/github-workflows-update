// Copyright (C) 2022 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! [`Error`] and [`Result`] types.

use reqwest;
use serde_json;

use thiserror;

pub type Result<T, E = Error> = core::result::Result<T, E>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    // Own errors
    #[error("updater not found for {0}")]
    UpdaterNotFound(String),
    #[error("unable to parse version in {0}")]
    VersionParsing(String),
    #[error("{1} while getting {0}")]
    HttpError(String, reqwest::StatusCode),
    #[error("{0} while parsing json")]
    JsonParsing(String),

    // Forwarded errors
    #[error(transparent)]
    JsonError(#[from] serde_json::Error),
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
}
