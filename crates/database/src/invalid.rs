use crate::file;
use crate::Result;
use std::{collections::HashSet, path::Path};

pub fn save_as<P>(assets: HashSet<String>, dir: P) -> Result<HashSet<String>>
where
    P: AsRef<Path>,
{
    let path = dir.as_ref().join("invalid");

    let mut unique_lines = file::unique_lines(&path)?;
    let news = &assets - &unique_lines;

    unique_lines.extend(assets);

    file::save(&unique_lines, &path)?;

    Ok(news)
}
