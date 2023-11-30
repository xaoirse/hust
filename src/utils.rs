use crate::Result;

use memchr::memmem;
use memmap2::Mmap;
use std::{
    ffi::OsString,
    fs::OpenOptions,
    io::{Read, Seek, Write},
    ops::Deref,
    os::unix::ffi::OsStrExt,
    path::PathBuf,
};

/// Trait to allow trimming ascii whitespace from a &[u8].
pub trait TrimAsciiWhitespace {
    /// Trim ascii whitespace (based on `is_ascii_whitespace()`) from the
    /// start and end of a slice.
    fn trim_ascii_whitespace(&self) -> &[u8];
}

impl<T: Deref<Target = [u8]>> TrimAsciiWhitespace for T {
    fn trim_ascii_whitespace(&self) -> &[u8] {
        let from = match self.iter().position(|x| !x.is_ascii_whitespace()) {
            Some(i) => i,
            None => return &self[0..0],
        };
        let to = self.iter().rposition(|x| !x.is_ascii_whitespace()).unwrap();
        &self[from..=to]
    }
}

pub trait Memfind<'a> {
    fn find(&'a self, needles: &[OsString]) -> Vec<&'a [u8]>;
}

impl<'a> Memfind<'a> for Mmap {
    fn find(&'a self, needles: &[OsString]) -> Vec<&[u8]> {
        self.split(|c| c == &b'\n')
            .filter(|l| !l.trim_ascii_whitespace().is_empty())
            .filter(|line| {
                needles.is_empty()
                    || needles
                        .iter()
                        .any(|needle| memmem::find(line, needle.as_bytes()).is_some())
            })
            .collect()
    }
}

pub fn append(path: PathBuf, str: &str) -> Result<()> {
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

    Ok(file.write_all(str.as_bytes())?)
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

        super::append(path.into(), "b").unwrap();

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

        super::append(path.into(), "b").unwrap();

        let file = std::fs::read_to_string(path).unwrap();

        assert_eq!(file, "a\nb\n");
    }
}
