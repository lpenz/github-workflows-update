// Copyright (C) 2022 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use std::fmt;

use versions;

/// Wrapper for the underlying external Version type
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Default, Clone)]
pub struct Version {
    pub v: versions::Version,
}

impl Version {
    pub fn new(s: &str) -> Option<Version> {
        versions::Version::new(s).map(|v| Version { v })
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.v)
    }
}
