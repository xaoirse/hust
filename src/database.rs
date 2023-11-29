use cidr_utils::{cidr::IpCidr, utils::IpCidrCombiner};
use fancy_regex::Regex;
use itertools::Itertools;
use std::{
    collections::HashSet,
    ffi::OsString,
    fmt::Display,
    fs::{File, OpenOptions},
    io::{Read, Seek, Write},
    os::unix::ffi::OsStrExt,
    path::{Path, PathBuf},
};

use crate::{utils::TrimAsciiWhitespace, Result};

#[derive(Debug)]
pub struct DataBase {
    ip: (File, IpCidrCombiner),
    domain: (File, HashSet<OsString>),
    other: (File, HashSet<OsString>),
}

impl DataBase {
    pub fn init(path: PathBuf, program: &OsString) -> Result<Self> {
        let path = path.join(program);
        std::fs::create_dir_all(&path)?;

        let (ip, ips) = file_lines(path.join("ip"))?;
        let (domain, domains) = file_lines(path.join("domain"))?;
        let (other, others) = file_lines(path.join("other"))?;

        let mut db = Self {
            ip: (ip, IpCidrCombiner::new()),
            domain: (domain, HashSet::new()),
            other: (other, HashSet::new()),
        };

        let _ = db.import(&ips);
        let _ = db.import(&domains);
        let _ = db.import(&others);

        Ok(db)
    }

    pub fn import<'a>(&mut self, args: &'a [OsString]) -> Result<Vec<&'a OsString>> {
        let r = Regex::new(r"^(?:(?!-|[^.]+_)[A-Za-z0-9-_]{1,63}(?<!-)(?:\.|$)){2,}$")?;

        let mut new = Vec::new();

        for arg in args {
            if let Ok(ip) =
                IpCidr::try_from(unsafe { std::str::from_utf8_unchecked(arg.as_bytes()) })
            {
                if !self.ip.1.contains(ip.first_as_ip_addr())
                    && !self.ip.1.contains(ip.last_as_ip_addr())
                {
                    self.ip.1.push(ip);
                    new.push(arg);
                }
            } else if r.is_match(&arg.to_string_lossy())? {
                if self.domain.1.insert(arg.clone()) {
                    new.push(arg);
                }
            } else if self.other.1.insert(arg.clone()) {
                new.push(arg)
            }
        }

        Ok(new)
    }

    pub fn write(&mut self) -> Result<()> {
        force_write(&mut self.ip.0, self.ip.1.get_ipv4_cidrs())?;
        force_write(&mut self.ip.0, self.ip.1.get_ipv6_cidrs())?;

        force_write(
            &mut self.domain.0,
            self.domain.1.iter().map(|s| s.to_string_lossy()),
        )?;

        force_write(
            &mut self.other.0,
            self.other.1.iter().map(|s| s.to_string_lossy()),
        )?;

        Ok(())
    }
}

fn file_lines(path: impl AsRef<Path>) -> Result<(File, Vec<OsString>)> {
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(path)?;

    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;

    Ok((
        file,
        buf.trim_ascii_whitespace()
            .split(|c| c == &b'\n')
            .map(|line| unsafe {
                OsString::from_encoded_bytes_unchecked(line.trim_ascii_whitespace().to_vec())
            })
            .filter(|l| !l.is_empty())
            .collect(),
    ))
}

fn force_write<I, T>(file: &mut File, iter: I) -> Result<()>
where
    I: IntoIterator<Item = T>,
    T: Display,
{
    file.set_len(0)?;
    file.seek(std::io::SeekFrom::Start(0))?;

    Ok(file.write_all(iter.into_iter().join("\n").as_bytes())?)
}
