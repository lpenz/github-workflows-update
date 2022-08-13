// Copyright (C) 2022 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! [`Error`] and [`Result`] types.

use reqwest;
use serde_json;
use url;

use thiserror;

pub type Result<T, E = Error> = core::result::Result<T, E>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("could not parse resource {0}")]
    ResourceParseError(String),
    #[error("unable to parse version in {0}")]
    VersionParsing(String),
    #[error("{1} while getting {0}")]
    HttpError(url::Url, reqwest::StatusCode),
    #[error("{0} while parsing json")]
    JsonParsing(String),

    // Forwarded errors
    #[error(transparent)]
    JsonError(#[from] serde_json::Error),
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
    #[error(transparent)]
    UrlParseError(#[from] url::ParseError),
}
