use std::cmp::Ordering;
use std::collections::BTreeMap;

use cu::pre::*;

use crate::hmgr;

/// Wrapper for parsing version number
#[derive(PartialEq, Display, DebugCustom)]
#[display("{}", self.0)]
#[debug("{}", self.0)]
pub struct Version<'a>(pub &'a str);
impl Version<'_> {
    /// Return true if self is less than other, or self is not comparable with other
    #[inline(always)]
    pub fn lt(&self, other: impl AsRef<str>) -> bool {
        !matches!(self.compare(other), Some(Ordering::Less) | None)
    }
    /// Compare 2 versions
    #[inline(always)]
    pub fn compare(&self, other: impl AsRef<str>) -> Option<Ordering> {
        self.compare_internal(other.as_ref())
    }
    fn compare_internal(&self, other: &str) -> Option<Ordering> {
        if self.0 == other {
            return Some(Ordering::Equal);
        }
        let self_parts = self.0.trim().split(['.', '-', '_']).collect::<Vec<_>>();
        let other_parts = other.trim().split(['.', '-', '_']).collect::<Vec<_>>();
        for (s, o) in std::iter::zip(&self_parts, &other_parts) {
            let s = s.trim();
            let o = o.trim();
            if s == o {
                continue;
            }
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
#[derive(Clone, Copy)]
pub struct VersionCache {
    id: &'static str,
    expected: &'static str,
}
impl VersionCache {
    pub const fn new(id: &'static str, expected: &'static str) -> Self {
        Self { id, expected }
    }
    /// Check if the cached version is the same as expected
    pub fn is_uptodate(self) -> cu::Result<bool> {
        let cached_version = get_cached_version(self.id)?;
        let is_uptodate = cached_version.as_deref() == Some(self.expected);
        cu::debug!("version cache not up to date: {}", self.id);
        Ok(is_uptodate)
    }
    /// Set the cached version to be the expected
    pub fn update(self) -> cu::Result<()> {
        set_cached_version(self.id, self.expected)
    }
}

fn get_cached_version(identifier: &str) -> cu::Result<Option<String>> {
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

fn set_cached_version(identifier: &str, version: &str) -> cu::Result<()> {
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
