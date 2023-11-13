use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::{fs::OpenOptions, path::Path};

#[derive(Debug)]
pub struct Assets {
    ips: HashSet<String>,
    domains: HashSet<String>,
}

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

impl TryFrom<Vec<String>> for Assets {
    type Error = Box<dyn std::error::Error>;

    fn try_from(value: Vec<String>) -> Result<Self> {
        Ok(Self {
            ips: HashSet::from_iter(value),
            domains: HashSet::new(),
        })
    }
}

impl Assets {
    pub fn save_as<P>(self, path: P) -> Result<HashSet<String>>
    where
        P: AsRef<Path>,
    {
        std::fs::create_dir_all(&path)?;

        save_as(self.ips, path.as_ref().join("ip"))
            .and(save_as(self.domains, path.as_ref().join("domain")))
    }
}

fn save_as<P>(assets: HashSet<String>, path: P) -> Result<HashSet<String>>
where
    P: AsRef<Path>,
{
    let (file, unique_lines) = unique_lines(path)?;

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

fn unique_lines<P>(path: P) -> Result<(File, HashSet<String>)>
where
    P: AsRef<Path>,
{
    let file = OpenOptions::new()
        .read(true)
        .append(true)
        .create(true)
        .open(path)
        .unwrap();

    // Create a hash set to store the unique lines in the file.
    let mut unique_lines = HashSet::new();

    // Read the file and add each line to the hash set.
    let mut reader = BufReader::new(&file);
    let mut line_buffer = String::new();
    while reader.read_line(&mut line_buffer)? > 0 {
        unique_lines.insert(line_buffer.trim().to_string());
        line_buffer.clear();
    }
    Ok((file, unique_lines))
}

pub fn _add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = _add(2, 2);
        assert_eq!(result, 4);
    }
}
