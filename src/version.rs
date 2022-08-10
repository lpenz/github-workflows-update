// Copyright (C) 2022 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

//! Type wrapper for versions; currently using [`semver`]
//! with [`lenient_semver`]

use std::fmt;

use lenient_semver;
use semver;

/// Wrapper for the underlying external Version type
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Default, Hash)]
pub struct Version {
    /// The parsed, full-typed version
    pub version: Option<semver::Version>,
    /// The original parsed string
    pub string: String,
}

impl Version {
    pub fn new(s: &str) -> Option<Version> {
        Some(Version {
            version: lenient_semver::parse(s).ok(),
            string: String::from(s),
        })
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.string)
    }
}
