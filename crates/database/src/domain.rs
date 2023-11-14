use crate::file;
use crate::Result;
use std::{collections::HashSet, io::BufWriter, io::Write, path::Path};

pub fn save_as<P>(assets: HashSet<String>, path: P) -> Result<HashSet<String>>
where
    P: AsRef<Path>,
{
    let (file, unique_lines) = file::unique_lines(path.as_ref().join("domain"))?;

    // Truncate the file to remove any duplicate lines.
    file.set_len(0)?;

    // Write the unique lines back to the file.
    let mut writer = BufWriter::new(file);

    unique_lines
        .iter()
        .map(|line| write!(writer, "{}\n", line).map_err(|err| err.into()))
        .collect::<Result<()>>()?;

    let news = &assets - &unique_lines;

    news.into_iter()
        .map(|line| {
            write!(writer, "{}\n", line)
                .map_err(|err| err.into())
                .and_then(|_| Ok(line))
        })
        .collect()
}
