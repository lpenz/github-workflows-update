// Copyright (C) 2022 Leandro Lisboa Penz <lpenz@lpenz.org>
// This file is subject to the terms and conditions defined in
// file 'LICENSE', which is part of this source code package.

use anyhow::{anyhow, Context, Result};
use serde_yaml::Value;
use std::io;
use versions::Version;

use crate::entity::Entity;

macro_rules! regex {
    ($re:literal $(,)?) => {{
        static RE: once_cell::sync::OnceCell<regex::Regex> = once_cell::sync::OnceCell::new();
        RE.get_or_init(|| regex::Regex::new($re).unwrap())
    }};
}

pub fn reference_parse_version(reference: &str) -> Option<(String, Version)> {
    let re_docker = regex!(r"^(?P<resource>docker://[^:]+):v?(?P<version>[0-9.]+)$");
    let m = re_docker.captures(reference)?;
    Some((
        m.name("resource").unwrap().as_str().into(),
        Version::new(m.name("version").unwrap().as_str())?,
    ))
}

pub fn buf_parse(r: impl io::BufRead) -> Result<Vec<Entity>> {
    let data: serde_yaml::Mapping =
        serde_yaml::from_reader(r).with_context(|| anyhow!("error in serde_yaml"))?;
    let jobs = data
        .get(&Value::String("jobs".into()))
        .ok_or_else(|| anyhow!("jobs entry not found"))?
        .as_mapping()
        .ok_or_else(|| anyhow!("invalid type for jobs entry"))?;
    let mut ret = vec![];
    for (jobname_, job) in jobs {
        if let Some(steps) = job.get(&Value::String("steps".into())) {
            let steps = steps
                .as_sequence()
                .ok_or_else(|| anyhow!("invalid type for steps entry"))?;
            let jobname = jobname_
                .as_str()
                .ok_or_else(|| anyhow!("invalid type for job key"))?;
            for step in steps {
                if let Some(uses) = step.get(&Value::String("uses".into())) {
                    let reference = uses
                        .as_str()
                        .ok_or_else(|| anyhow!("invalid type for uses entry"))?;
                    if let Some((resource, version)) = reference_parse_version(reference) {
                        let v = Entity {
                            job: String::from(jobname),
                            reference: reference.into(),
                            resource,
                            version,
                        };
                        ret.push(v);
                    }
                }
            }
        }
    }
    Ok(ret)
}

#[test]
fn test_parse() -> Result<()> {
    let s = r"
---
name: test
jobs:
  omnilint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: docker://lpenz/omnilint:0.4
      - run: ls
";
    buf_parse(s.as_bytes())?;
    Ok(())
}
