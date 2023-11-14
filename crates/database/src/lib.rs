mod domain;
mod file;
mod ip;
mod log;

use std::collections::HashSet;
use std::path::Path;
use url;

#[derive(Debug, Default)]
pub struct DataBase {
    ips: HashSet<String>,
    domains: HashSet<String>,
}

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

impl TryFrom<Vec<String>> for DataBase {
    type Error = Box<dyn std::error::Error>;

    fn try_from(value: Vec<String>) -> Result<Self> {
        let mut assests = Self::default();
        for val in value {
            if val.parse::<cidr_utils::cidr::IpCidr>().is_ok() {
                assests.ips.insert(val);
            } else if url::Host::parse(&val).is_ok() {
                assests.domains.insert(val);
            }
        }
        Ok(assests)
    }
}

impl DataBase {
    pub fn save_as<P>(self, path: P) -> Result<HashSet<String>>
    where
        P: AsRef<Path>,
    {
        std::fs::create_dir_all(&path)?;

        ip::save_as(self.ips, &path).and(domain::save_as(self.domains, &path))
    }
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
