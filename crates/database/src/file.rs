use std::{
    collections::HashSet,
    fmt::Display,
    fs::OpenOptions,
    io::{BufRead, BufReader, BufWriter, Write},
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

/// ## [Method-call expressions](https://doc.rust-lang.org/stable/reference/expressions/method-call-expr.html#method-call-expressions)
///
/// > ...For instance, if the receiver has type Box<[i32;2]>,
/// > then the candidate types will be Box<[i32;2]>, &Box<[i32;2]>, &mut Box<[i32;2]>...
///
/// So, here we want (&V).into_iter(), we write bellow code and it
/// automatically dereferenced or borrowed in order to call a method.
///
/// For example: `(*iter).into_iter()` it works.
/// And this example too:
/// ```
/// fn show<I, D>(iter: D)
/// where
///     D: Deref<Target = I>,
///     for<'a> &'a &'a &'a I: IntoIterator<Item = &'a &'a String>,
/// {
///     (&&(*iter)).into_iter()
/// }
/// ```
/// But not
/// ```
///     (&(*iter)).into_iter()
/// ```
pub fn save<I, P, T, V>(iter: I, path: P) -> Result<()>
where
    I: Deref<Target = V>,

    for<'a> &'a V: IntoIterator<Item = &'a T>,
    T: Display,
    P: AsRef<Path>,
{
    let file = OpenOptions::new()
        .read(true)
        .create(true)
        .append(true)
        .open(path)
        .unwrap();

    // Truncate the file to remove any duplicate lines.
    file.set_len(0)?;

    let mut writer = BufWriter::new(file);

    iter.into_iter()
        .map(|line| write!(writer, "{}\n", line).map_err(|err| err.into()))
        .collect::<Result<()>>()
}
