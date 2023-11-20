use std::{
    collections::HashSet,
    fmt::Display,
    fs::OpenOptions,
    io::{BufRead, BufReader, BufWriter, Read, Seek, Write},
    ops::Deref,
    path::Path,
};

use crate::Result;

pub fn unique_lines<P>(path: P) -> Result<HashSet<String>>
where
    P: AsRef<Path>,
{
    // Create a hash set to store the unique lines in the file.
    let mut unique_lines = HashSet::new();

    // If file exists read file else return empty HashSet
    if let Ok(file) = OpenOptions::new().read(true).open(path) {
        // Read the file and add each line to the hash set.
        let mut reader = BufReader::new(&file);
        let mut line_buffer = String::new();
        while reader.read_line(&mut line_buffer)? > 0 {
            unique_lines.insert(line_buffer.trim().to_string());
            line_buffer.clear();
        }
    }

    Ok(unique_lines)
}

pub fn save<I, P, T, V>(path: P, iter: I) -> Result<()>
where
    I: Deref<Target = V>,

    for<'a> &'a V: IntoIterator<Item = &'a T>,
    T: Display,
    P: AsRef<Path>,
{
    let file = OpenOptions::new()
        .truncate(true)
        .write(true)
        .create(true)
        .open(path)
        .unwrap();

    let mut writer = BufWriter::new(file);

    iter.into_iter()
        .try_for_each(|line| writeln!(writer, "{}", line).map_err(|err| err.into()))
}

pub fn append<I, P, T, V>(iter: I, path: P) -> Result<()>
where
    I: Deref<Target = V>,

    for<'a> &'a V: IntoIterator<Item = &'a T>,
    T: Display,
    P: AsRef<Path>,
{
    let mut file = OpenOptions::new()
        .read(true)
        .append(true)
        .create(true)
        .open(path)
        .unwrap();

    let mut buf = [0; 1];

    if file.seek(std::io::SeekFrom::End(-1)).is_ok()
        && file.read_exact(&mut buf).is_ok()
        && buf[0] != b'\n'
    {
        file.write_all(b"\n")?;
    }

    let mut writer = BufWriter::new(file);

    iter.into_iter()
        .try_for_each(|line| writeln!(writer, "{}", line).map_err(|err| err.into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn append_1() {
        let path = "/tmp/hust.test";
        let mut file = OpenOptions::new()
            .truncate(true)
            .write(true)
            .create(true)
            .open(path)
            .unwrap();

        file.write_all(b"a").unwrap();

        super::append(&["b"], path).unwrap();

        let file = std::fs::read_to_string(path).unwrap();

        assert_eq!(file, "a\nb\n");
    }

    #[test]
    fn append_2() {
        let path = "/tmp/hust.test";
        let mut file = OpenOptions::new()
            .truncate(true)
            .write(true)
            .create(true)
            .open(path)
            .unwrap();

        file.write_all(b"a\n").unwrap();

        super::append(&["b"], path).unwrap();

        let file = std::fs::read_to_string(path).unwrap();

        assert_eq!(file, "a\nb\n");
    }
}
