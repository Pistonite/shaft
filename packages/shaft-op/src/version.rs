use std::cmp::Ordering;

#[derive(PartialEq)]
pub struct Version<'a>(pub &'a str);
impl<'a> PartialOrd for Version<'a> {
    #[inline(always)]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.partial_cmp(&other.0)
    }
}
impl<'a> PartialEq<&str> for Version<'a> {
    #[inline(always)]
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}
impl<'a> PartialOrd<&str> for Version<'a> {
    fn partial_cmp(&self, other: &&str) -> Option<Ordering> {
        if self.0 == *other {
            return Some(Ordering::Equal);
        }
        let self_parts = self.0.trim().split('.').collect::<Vec<_>>();
        let other_parts = other.trim().split('.').collect::<Vec<_>>();
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
impl<'a> PartialOrd<String> for Version<'a> {
    #[inline(always)]
    fn partial_cmp(&self, other: &String) -> Option<Ordering> {
        self.partial_cmp(&other.as_str())
    }
}
