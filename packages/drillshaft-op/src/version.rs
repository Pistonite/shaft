pub trait VersionNumber {
    fn as_str(&self) -> &str;
    fn to_parts(&self) -> Vec<&str> {
        self.as_str().split('.').collect()
    }
    fn is_version_same_or_higher_than(&self, other: &str) -> bool {
        if self.as_str() == other {
            return true;
        }
        let self_parts = self.to_parts();
        let other_parts = other.to_parts();

        for (s, o) in std::iter::zip(&self_parts, &other_parts) {
            match (cu::parse::<u64>(s), cu::parse::<u64>(o)) {
                (Ok(s), Ok(o)) => {
                    // both are version numbers
                    match s.cmp(&o) {
                        std::cmp::Ordering::Less => return false,
                        std::cmp::Ordering::Greater => return true,
                        std::cmp::Ordering::Equal => {}
                    }
                }
                // not comparable
                _ => return false,
            }
        }

        if self_parts.len() > other_parts.len() {
            // self is longer, probably a higher version
            return true;
        }

        // equal
        return true;
    }
}

impl VersionNumber for str {
    #[inline(always)]
    fn as_str(&self) -> &str {
        self
    }
}

impl VersionNumber for String {
    #[inline(always)]
    fn as_str(&self) -> &str {
        self.as_str()
    }
}
