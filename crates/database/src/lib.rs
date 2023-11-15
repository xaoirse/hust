mod domain;
mod file;
mod invalids;
mod ip;
mod log;

use addr::parse_dns_name;
use std::collections::HashSet;
use std::path::Path;

#[derive(Debug, Default)]
pub struct DataBase {
    ips: HashSet<String>,
    domains: HashSet<String>,
    invalids: HashSet<String>,
}

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

impl From<Vec<String>> for DataBase {
    fn from(value: Vec<String>) -> Self {
        let mut assests = Self::default();
        for val in value {
            if val.parse::<cidr_utils::cidr::IpCidr>().is_ok() {
                assests.ips.insert(val);
            } else if parse_dns_name(&val).is_ok_and(|n| n.is_icann()) {
                assests.domains.insert(val);
            } else {
                assests.invalids.insert(val);
            }
        }
        assests
    }
}

impl DataBase {
    pub fn save_as<P>(self, path: P) -> Result<HashSet<String>>
    where
        P: AsRef<Path>,
    {
        std::fs::create_dir_all(&path)?;

        let mut result = HashSet::new();

        result.extend(ip::save_as(self.ips, &path)?);
        result.extend(domain::save_as(self.domains, &path)?);
        result.extend(invalids::save_as(self.invalids, &path)?);

        Ok(result)
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
