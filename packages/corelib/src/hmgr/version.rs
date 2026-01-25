use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::fmt;

use cu::pre::*;

use crate::hmgr;

/// Wrapper for parsing version number
#[derive(PartialEq)]
pub struct Version<'a>(pub &'a str);
impl<'a> PartialOrd<&str> for Version<'a> {
    fn partial_cmp(&self, other: &&str) -> Option<Ordering> {
        if self.0 == *other {
            return Some(Ordering::Equal);
        }
        let self_parts = self.0.trim().split(['.', '-', '_']).collect::<Vec<_>>();
        let other_parts = other.trim().split(['.', '-', '_']).collect::<Vec<_>>();
        for (s, o) in std::iter::zip(&self_parts, &other_parts) {
            match (cu::parse::<u64>(s), cu::parse::<u64>(o)) {
                (Ok(s), Ok(o)) => {
                    // both are version numbers
                    match s.cmp(&o) {
                        Ordering::Less => return Some(Ordering::Less),
                        Ordering::Greater => return Some(Ordering::Greater),
                        Ordering::Equal => {}
                    }
                }
                // not comparable
                _ => return None,
            }
        }

        self.0.len().partial_cmp(&other.len())
    }
}
impl<'a> PartialEq<String> for Version<'a> {
    #[inline(always)]
    fn eq(&self, other: &String) -> bool {
        self.0 == other
    }
}
impl<'a> PartialEq<&str> for Version<'a> {
    #[inline(always)]
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}
impl<'a> PartialOrd<String> for Version<'a> {
    #[inline(always)]
    fn partial_cmp(&self, other: &String) -> Option<Ordering> {
        self.partial_cmp(&other.as_str())
    }
}
impl<'a> PartialOrd for Version<'a> {
    #[inline(always)]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.partial_cmp(&other.0)
    }
}
impl<'a> fmt::Debug for Version<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.0, f)
    }
}
impl<'a> fmt::Display for Version<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.0, f)
    }
}

pub fn get_cached_version(identifier: &str) -> cu::Result<Option<String>> {
    let path = hmgr::paths::version_cache_json();
    if !path.exists() {
        return Ok(None);
    }
    let file = cu::check!(cu::fs::read_string(&path), "error reading version cache")?;
    let map = match json::parse::<BTreeMap<String, String>>(&file) {
        Ok(x) => x,
        Err(e) => {
            cu::warn!("failed to parse version cache: {e:?}");
            return Ok(None);
        }
    };
    Ok(map.get(identifier).cloned())
}

pub fn set_cached_version(identifier: &str, version: &str) -> cu::Result<()> {
    let path = hmgr::paths::version_cache_json();
    let mut map = if !path.exists() {
        BTreeMap::new()
    } else {
        let file = cu::check!(cu::fs::read_string(&path), "error reading version cache")?;
        match json::parse::<BTreeMap<String, String>>(&file) {
            Ok(x) => x,
            Err(e) => {
                cu::warn!("failed to parse version cache: {e:?}");
                Default::default()
            }
        }
    };
    map.insert(identifier.to_string(), version.to_string());
    cu::fs::write_json_pretty(&path, &map)?;
    Ok(())
}
