use std::collections::BTreeSet;

use crate::hmgr::Item;
use crate::hmgr::item::ItemEntry;

pub fn rebuild_user_path(items: &[ItemEntry]) -> cu::Result<(String, bool)> {
    use std::fmt::Write as _;

    let current_paths = cu::env_var("PATH")?;
    let current_paths: BTreeSet<_> = current_paths
        .split(':')
        .map(|x| x.trim().to_string())
        .collect();

    let mut reinvocation_needed = false;
    let mut controlled_paths = vec![];
    for entry in items {
        let Item::UserPath(p) = &entry.item else {
            continue;
        };
        controlled_paths.push(p);
        if !current_paths.contains(p) {
            cu::debug!("itemmgr: reinvocation because of path: adding '{p}'");
            reinvocation_needed = true;
        }
    }
    let mut seen = BTreeSet::new();
    let mut out = String::new();

    // on non-Windows, simply append to existing $PATH in the shell
    let _ = write!(out, "$SHAFT_HOME/bin");
    // latest added path go to the front
    for p in controlled_paths.iter().rev() {
        let p = p.trim();
        if p.is_empty() {
            continue;
        }
        if seen.insert(p) {
            let _ = write!(out, ":{p}");
        }
    }
    out.push_str(":$PATH");

    Ok((out, reinvocation_needed))
}
