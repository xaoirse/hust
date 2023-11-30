use cidr_utils::{cidr::IpCidr, utils::IpCidrCombiner};
use fancy_regex::Regex;
use std::{
    collections::HashSet,
    ffi::OsString,
    fs::{File, OpenOptions},
    os::unix::ffi::OsStrExt,
    path::Path,
    rc::Rc,
};

use crate::{
    utils::{file_lines, force_write},
    Result,
};

#[derive(Debug)]
pub struct DataBase {
    ip: (File, IpCidrCombiner),
    domain: (File, HashSet<Rc<OsString>>),
    other: (File, HashSet<Rc<OsString>>),
    pub new: (File, HashSet<Rc<OsString>>),
}

impl DataBase {
    pub fn init(path: &Path, program: &OsString) -> Result<Self> {
        let new = OpenOptions::new()
            .append(true)
            .create(true)
            .open(path.join("hust.log"))?;

        let path = path.join(program);
        std::fs::create_dir_all(&path)?;

        let (ip, ips) = file_lines(path.join("ip"))?;
        let (domain, domains) = file_lines(path.join("domain"))?;
        let (other, others) = file_lines(path.join("other"))?;

        let db = Self {
            ip: (ip, IpCidrCombiner::new()),
            domain: (domain, HashSet::new()),
            other: (other, HashSet::new()),
            new: (new, HashSet::new()),
        };

        Ok(db
            .import(ips, false)
            .import(domains, false)
            .import(others, false))
    }

    pub fn import(mut self, args: Vec<OsString>, new: bool) -> Self {
        let r = Regex::new(r"^(?:(?!-|[^.]+_)[A-Za-z0-9-_]{1,63}(?<!-)(?:\.|$)){2,}$").unwrap();

        for arg in args {
            let arg = Rc::new(arg);
            if let Ok(ip) =
                IpCidr::try_from(unsafe { std::str::from_utf8_unchecked(arg.as_bytes()) })
            {
                if !self.ip.1.contains(ip.first_as_ip_addr())
                    && !self.ip.1.contains(ip.last_as_ip_addr())
                {
                    self.ip.1.push(ip);
                    if new {
                        self.new.1.insert(arg);
                    }
                }
            } else if r.is_match(&arg.to_string_lossy()).unwrap() {
                if self.domain.1.insert(Rc::clone(&arg)) && new {
                    self.new.1.insert(arg);
                }
            } else if self.other.1.insert(Rc::clone(&arg)) && new {
                self.new.1.insert(arg);
            }
        }

        self
    }

    pub fn write(&mut self) -> Result<()> {
        force_write(
            &mut self.ip.0,
            self.ip
                .1
                .get_ipv4_cidrs()
                .iter()
                .map(|c| c.to_string())
                .chain(self.ip.1.get_ipv6_cidrs().iter().map(|c| c.to_string())),
        )?;

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
