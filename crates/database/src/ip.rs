use crate::{file, invalids, Result};
use cidr_utils::{cidr::Ipv4Cidr, utils::Ipv4CidrCombiner};
use std::{collections::HashSet, path::Path};

pub fn save_as<P>(assets: HashSet<String>, dir: P) -> Result<HashSet<String>>
where
    P: AsRef<Path>,
{
    let path = dir.as_ref().join("ip");

    let unique_lines = file::unique_lines(&path)?;
    let mut invalids = HashSet::new();
    let mut combiner = Ipv4CidrCombiner::new();

    let news = &assets - &unique_lines;

    for s in unique_lines.into_iter().chain(assets.into_iter()) {
        match Ipv4Cidr::from_str(&s) {
            Ok(ip) => combiner.push(ip),
            Err(_) => {
                invalids.insert(s);
            }
        }
    }

    invalids::save_as(invalids, dir)?;
    file::save(combiner, &path)?;

    Ok(news)
}
