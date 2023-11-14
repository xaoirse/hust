use crate::{file, Result};
use cidr_utils::{cidr::Ipv4Cidr, utils::Ipv4CidrCombiner};
use std::{collections::HashSet, io::BufWriter, io::Write, path::Path};

pub fn save_as<P>(assets: HashSet<String>, path: P) -> Result<HashSet<String>>
where
    P: AsRef<Path>,
{
    let (file, unique_lines) = file::unique_lines(path.as_ref().join("ip"))?;

    let mut combiner = Ipv4CidrCombiner::new();

    for l in &unique_lines {
        combiner.push(Ipv4Cidr::from_str(l)?);
    }
    for l in &assets {
        combiner.push(Ipv4Cidr::from_str(l)?);
    }

    // Truncate the file to remove any duplicate lines.
    file.set_len(0)?;

    // Write the unique lines back to the file.
    let mut writer = BufWriter::new(file);

    combiner
        .iter()
        .map(|line| write!(writer, "{}\n", line).map_err(|err| err.into()))
        .collect::<Result<()>>()?;

    Ok(&assets - &unique_lines)
}
