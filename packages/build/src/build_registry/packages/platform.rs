use std::collections::BTreeMap;

use cu::pre::*;
use enumset::{EnumSet, EnumSetType, enum_set};
use itertools::Itertools;

#[derive(Debug, PartialOrd, Ord, Display, EnumSetType)]
pub enum Target {
    #[display("_win_x64")]
    WindowsX64,
    #[display("_win_arm")]
    WindowsArm,
    #[display("_linux-pacman_x64")]
    LinuxPacmanX64,
    #[display("_linux-apt_x64")]
    LinuxAptX64,
    #[display("_mac_arm")]
    MacosArm,
}
pub type TargetSet = EnumSet<Target>;
impl Target {
    pub fn parse(spec: &str) -> cu::Result<(&str, TargetSet)> {
        if spec.is_empty() {
            cu::bail!("package name cannot be empty, from spec: {spec}");
        }
        let (name, suffix) = match spec.find('_') {
            None => {
                return Ok((spec, enum_set!()));
            }
            Some(i) => {
                spec.split_at(i)
            }
        };
        if name.is_empty() {
            cu::bail!("package name cannot be empty, from spec: {spec}");
        }
        let targets = cu::check!(Self::parse_target_suffix(suffix), "failed to parse the target from package spec: {spec}")?;
        Ok((name, targets))
    }

    fn parse_target_suffix(suffix: &str) -> cu::Result<TargetSet> {
        let targets = match suffix {
            "_win" => Self::win(),
            "_win_x64" => Self::WindowsX64.into(),
            "_win_arm" => Self::WindowsArm.into(),
            "_linux" => Self::linux(),
            "_linux_x64" => Self::linux_x64(),
            // "_linux_arm" => { }
            "_linux-pacman" => Self::linux_pacman(),
            "_linux-pacman_x64" => Self::LinuxPacmanX64.into(),
            // "_linux-pacman_arm" => { }
            "_linux-apt" => Self::linux_apt(),
            "_linux-apt_x64" => Self::LinuxAptX64.into(),
            // "_linux-apt_arm" => { }
            "_mac" => Self::mac(),
            "_mac_arm" => Self::MacosArm.into(),

            _ => {
                if suffix.contains("armv7") {
                    cu::bail!("armv7 is not supported");
                }
                if suffix.contains("linux") {
                    if suffix.contains("arm") || suffix.contains("aarch") {
                        cu::bail!("only x64 is supported on Linux");
                    }
                }
                let mut suggestion: Option<String> = None;
                if suffix.contains("windows") {
                    suggestion = Some(suggestion.as_deref().unwrap_or(suffix).replace("windows", "win"));
                }
                if suffix.contains("aarch64") {
                    suggestion = Some(suggestion.as_deref().unwrap_or(suffix).replace("aarch64", "arm"));
                }
                if suffix.contains("armv8") {
                    suggestion = Some(suggestion.as_deref().unwrap_or(suffix).replace("armv8", "arm"));
                }
                if suffix.contains("macos") {
                    suggestion = Some(suggestion.as_deref().unwrap_or(suffix).replace("macos", "mac"));
                }
                match suggestion {
                    None => {
                        cu::bail!("unrecognized target: {suffix}");
                    }
                    Some(x) => {
                        cu::bail!("unrecognized target: {suffix}; try: {x}");
                    }
                }
            }
        };
        Ok(targets)
    }
    /// *_win (includes *_win_x64 and *_win_arm)
    pub fn win() -> TargetSet {
        enum_set! { Self::WindowsX64 | Self::WindowsArm }
    }
    /// *_linux (includes *_linux-pacman_x64 and *_linux-apt_x64)
    pub fn linux() -> TargetSet {
        enum_set! { Self::LinuxPacmanX64 | Self::LinuxAptX64 }
    }
    /// *_linux_x64 (includes *_linux-pacman_x64 and *_linux-apt_x64)
    pub fn linux_x64() -> TargetSet {
        Self::linux()
    }
    /// *_linux-pacman (includes *_linux-pacman_x64 )
    pub fn linux_pacman() -> TargetSet {
        Self::LinuxPacmanX64.into()
    }
    /// *_linux-apt (includes *_linux-apt_x64 )
    pub fn linux_apt() -> TargetSet {
        Self::LinuxAptX64.into()
    }
    /// *_mac (includes *_mac_arm)
    pub fn mac() -> TargetSet {
        Self::MacosArm.into()
    }

    pub fn raw_cfg(self) -> &'static str {
        match self {
            Target::WindowsX64 => r#"all(windows, target_arch="x86_64")"#,
            Target::WindowsArm => r#"all(windows, target_arch="aarch64")"#,
            Target::LinuxPacmanX64 | Target::LinuxAptX64=> r#"all(target_os = "linux", target_arch="x86_64")"#,
            Target::MacosArm => r#"all(target_os = "macos", target_arch="aarch64")"#,
        }
    }

    pub fn combine_targets<T: Default + PartialEq>(mut tree: BTreeMap<Self, T>) -> Vec<(TargetSet, T)> {
        for t in TargetSet::all() {
            tree.entry(t).or_default();
        }
        let mut out = Vec::with_capacity(tree.len());
        while let Some((target, value)) = tree.iter().next() {
            let mut targets = TargetSet::new();
            targets.insert(*target);
            for (t, v) in &tree {
                if t == target {
                    continue;
                }
                if v == value {
                    targets.insert(*t);
                }
            }
            let target = *target;
            let value = tree.remove(&target).unwrap();
            out.push((targets, value));
            for t in targets {
                tree.remove(&t);
            }
        }
        out
    }
}

pub struct CfgAttr(pub TargetSet);
impl CfgAttr {
    pub fn attr(&self) -> String {
        format!("#[cfg({})]", self.expr())
    }
    pub fn expr(&self) -> String {
        if self.0.is_empty() {
            format!("not({})", Self(TargetSet::all()).expr())
        } else {
            format!("any({})", self.0.into_iter().map(Target::raw_cfg).join(", "))
        }
    }
}
