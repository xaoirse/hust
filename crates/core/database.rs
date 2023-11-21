use crate::file;

use addr::parse_dns_name;
use cidr_utils::{cidr::IpCidr, utils::IpCidrCombiner};
use clap::ArgEnum;
use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
    path::{Path, PathBuf},
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug)]
pub struct DataBase {
    rows: HashMap<Key, HashSet<String>>,
}
impl Default for DataBase {
    fn default() -> Self {
        let mut hm = HashMap::new();
        hm.insert(Key::Ip, HashSet::new());
        hm.insert(Key::Domain, HashSet::new());
        hm.insert(Key::Other, HashSet::new());
        Self { rows: hm }
    }
}

impl From<Vec<String>> for DataBase {
    fn from(values: Vec<String>) -> Self {
        let mut db = Self::default();

        for value in values {
            db.insert(value);
        }
        db
    }
}

#[derive(Debug, Eq, Hash, PartialEq, Clone, Copy, ArgEnum)]
pub enum Key {
    Ip,
    Domain,
    Other,
}

impl<T: AsRef<str>> From<T> for Key {
    fn from(value: T) -> Self {
        if value.as_ref().parse::<IpCidr>().is_ok() {
            Key::Ip
        } else if parse_dns_name(value.as_ref()).is_ok_and(|n| n.is_icann()) {
            Key::Domain
        } else {
            Key::Other
        }
    }
}

impl AsRef<Path> for Key {
    fn as_ref(&self) -> &Path {
        match self {
            Key::Ip => Path::new("ip"),
            Key::Domain => Path::new("domain"),
            Key::Other => Path::new("other"),
        }
    }
}

impl DataBase {
    pub fn save(self, path: PathBuf) -> Result<HashSet<String>> {
        std::fs::create_dir_all(&path)?;

        let mut db = Self::default();

        for typ in self.rows.keys() {
            db.import(path.join(typ));
        }

        let new = db.merge(self);

        db.write(path)?;

        Ok(new)
    }

    fn import(&mut self, path: PathBuf) {
        file::unique_lines(&path)
            .unwrap_or_default()
            .into_iter()
            .for_each(|value| {
                self.insert(value);
            });
    }

    fn insert(&mut self, value: String) -> bool {
        self.rows
            .entry(Key::from(&value))
            .or_default()
            .insert(value)
    }

    fn merge(&mut self, other: Self) -> HashSet<String> {
        let mut new: HashSet<String> = HashSet::new();
        for (key, cli_row) in other.rows {
            let file_row = self.rows.get_mut(&key).unwrap();

            if key != Key::Ip {
                new.extend(&cli_row - &file_row);
                file_row.extend(cli_row);
            } else {
                let mut combiner = IpCidrCombiner::new();

                for s in file_row.iter() {
                    if let Ok(ip) = IpCidr::from_str(s) {
                        if !(combiner.contains(ip.first_as_ip_addr())
                            && combiner.contains(ip.last_as_ip_addr()))
                        {
                            combiner.push(ip)
                        }
                    }
                }

                for s in cli_row.iter() {
                    if let Ok(ip) = IpCidr::from_str(s) {
                        if !(combiner.contains(ip.first_as_ip_addr())
                            && combiner.contains(ip.last_as_ip_addr()))
                        {
                            combiner.push(ip);
                            new.insert(s.to_string());
                        }
                    }
                }

                file_row.clear();

                file_row.extend(combiner.get_ipv4_cidrs().iter().map(|i| i.to_string()));
                file_row.extend(combiner.get_ipv6_cidrs().iter().map(|i| i.to_string()));
            }
        }

        new
    }

    fn write(&self, path: PathBuf) -> Result<()> {
        for (typ, data) in &self.rows {
            file::save(&path.join(typ), data)?;
        }
        Ok(())
    }
}
