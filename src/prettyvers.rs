// Copyright (C) 2022 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use anyhow::Result;

use crate::version::Version;

/// Wrapper that prints a vector of versions using the default version formatter
pub struct Versions<'a> {
    versions: &'a [Version],
}

impl<'a> Versions<'a> {
    pub fn new(versions: &[Version]) -> Versions {
        Versions { versions }
    }
}

impl<'a> std::fmt::Debug for Versions<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        let v = self
            .versions
            .iter()
            .map(|v| format!("{}", v))
            .collect::<Vec<_>>();
        write!(f, "{:?}", v)
    }
}
