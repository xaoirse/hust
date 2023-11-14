use std::{
    collections::HashSet,
    fs::{File, OpenOptions},
    io::{BufRead, BufReader},
    path::Path,
};

use crate::Result;

pub fn unique_lines<P>(path: P) -> Result<(File, HashSet<String>)>
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
