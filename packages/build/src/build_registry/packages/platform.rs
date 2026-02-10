use std::collections::BTreeMap;

use cu::pre::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Display)]
pub enum Platform {
    /// Unconditional (no suffix)
    #[display("* (any)")]
    Any,
    /// *_win, target_os = "windows"
    #[display("_win")]
    Windows,
    /// *_win-x64, target_os = "windows", target_arch = "x86_64"
    #[display("_win-x64")]
    WindowsX64,
    /// *_win-arm target_os = "windows", target_arch = "aarch64"
    #[display("_win-arm")]
    WindowsArm64,
    /// *_linux, target_os = "linux", linux_flavor = all
    #[display("_linux")]
    Linux,
    /// *_linux-pacman, target_os = "linux", linux_flavor = pacman
    #[display("_linux-pacman")]
    LinuxPacman,
    /// *_linux-apt, target_os = "linux", linux_flavor = apt
    #[display("_linux-apt")]
    LinuxApt,
    /// *_mac, target_os = "macos", target_arch = "aarch64"
    #[display("_mac")]
    Macos,
}
impl Platform {
    pub fn cfg_attr(self) -> String {
        let Some(inner) = self.inner_cfg() else {
            return "".to_string();
        };
        format!("#[cfg({inner})]")
    }
    pub fn cfg_attr_inverted<I: Iterator<Item = Self>>(set: I) -> Option<String> {
        let mut not_in_set = Self::Any
            .leaves()
            .iter()
            .map(|p| (*p, true))
            .collect::<BTreeMap<_, _>>();
        for platform in set {
            for leaf in platform.leaves() {
                not_in_set.insert(*leaf, false);
            }
        }
        Self::combine_leaves(&mut not_in_set);
        let mut parts = vec![];
        for (platform, not_in) in not_in_set {
            if !not_in {
                continue;
            }
            let Some(part) = platform.inner_cfg() else {
                // if "any" is not in set, it will be unconditional
                return Some("".to_string());
            };
            parts.push(part);
        }
        if parts.is_empty() {
            return None; // everything is covered in set
        }
        Some(format!("#[cfg(any({}))]", parts.join(",")))
    }
    fn inner_cfg(self) -> Option<&'static str> {
        match self {
            Platform::Any => None,
            Platform::Windows => Some("target_os=\"windows\""),
            Platform::WindowsX64 => Some("all(target_os=\"windows\",target_arch=\"x86_64\")"),
            Platform::WindowsArm64 => Some("all(target_os=\"windows\",target_arch=\"aarch64\")"),
            Platform::Linux | Platform::LinuxPacman | Platform::LinuxApt => {
                Some("target_os=\"linux\"")
            }
            Platform::Macos => Some("all(target_os=\"macos\",target_arch=\"aarch64\")"),
        }
    }
    pub fn leaves(self) -> &'static [Self] {
        match self {
            Platform::Any => &[
                Self::WindowsX64,
                Self::WindowsArm64,
                Self::LinuxPacman,
                Self::LinuxApt,
                Self::Macos,
            ],
            Platform::Windows => &[Self::WindowsX64, Self::WindowsArm64],
            Platform::WindowsX64 => &[Self::WindowsX64],
            Platform::WindowsArm64 => &[Self::WindowsArm64],
            Platform::Linux => &[Self::LinuxPacman, Self::LinuxApt],
            Platform::LinuxPacman => &[Self::LinuxPacman],
            Platform::LinuxApt => &[Self::LinuxApt],
            Platform::Macos => &[Self::Macos],
        }
    }
    pub fn combine_leaves<T: Default + PartialEq>(tree: &mut BTreeMap<Self, T>) {
        for p in Self::Any.leaves() {
            tree.entry(*p).or_default();
        }
        let mut should_try_any = true;
        let linux = tree.get(&Self::LinuxPacman);
        if tree.get(&Self::LinuxApt) == linux {
            tree.remove(&Self::LinuxApt);
            let linux = tree
                .remove(&Self::LinuxPacman)
                .expect("combine_leaves linux");
            tree.insert(Self::Linux, linux);
        } else {
            should_try_any = false;
        }
        let windows = tree.get(&Self::WindowsX64);
        if tree.get(&Self::WindowsArm64) == windows {
            tree.remove(&Self::WindowsArm64);
            let windows = tree
                .remove(&Self::WindowsX64)
                .expect("combine_leaves windows");
            tree.insert(Self::Windows, windows);
        } else {
            should_try_any = false;
        }

        if should_try_any {
            let the_any = tree.get(&Self::Linux);
            if tree.get(&Self::Windows) == the_any && tree.get(&Self::Macos) == the_any {
                let the_any = tree.remove(&Self::Linux).expect("combine_leaves any");
                tree.clear();
                tree.insert(Self::Any, the_any);
            }
        }
    }
    pub fn find_conflict<I: Iterator<Item = Self>>(self, mut current: I) -> Option<Platform> {
        match self {
            Self::Any => current.next(),
            Self::Windows => current.find(|p| {
                matches!(
                    p,
                    Self::Any | Self::Windows | Self::WindowsArm64 | Self::WindowsX64
                )
            }),
            Self::WindowsX64 => {
                current.find(|p| matches!(p, Self::Any | Self::Windows | Self::WindowsX64))
            }
            Self::WindowsArm64 => {
                current.find(|p| matches!(p, Self::Any | Self::Windows | Self::WindowsArm64))
            }
            Self::Linux => current.find(|p| {
                matches!(
                    p,
                    Self::Any | Self::Linux | Self::LinuxPacman | Self::LinuxApt
                )
            }),
            Self::LinuxPacman => {
                current.find(|p| matches!(p, Self::Any | Self::Linux | Self::LinuxPacman))
            }
            Self::LinuxApt => {
                current.find(|p| matches!(p, Self::Any | Self::Linux | Self::LinuxApt))
            }
            Self::Macos => current.find(|p| matches!(p, Self::Any | Self::Macos)),
        }
    }
    pub fn linux_flavors(self) -> &'static str {
        match self {
            Self::Any | Self::Linux => "corelib::opfs::LinuxFlavor::all()",
            Self::LinuxPacman => "enum_set!{ corelib::opfs::LinuxFlavor::Pacman }",
            Self::LinuxApt => "enum_set!{ corelib::opfs::LinuxFlavor::Apt }",
            _ => "corelib::opfs::LinuxFlavor::none()",
        }
    }
    pub fn linux_flavor(self) -> &'static str {
        match self {
            Self::LinuxPacman => "corelib::opfs::LinuxFlavor::Pacman",
            Self::LinuxApt => "corelib::opfs::LinuxFlavor::Apt",
            _ => "",
        }
    }
    pub fn is_linux_leaf(self) -> bool {
        Platform::Linux.leaves().contains(&self)
    }
    pub fn module_str(self) -> &'static str {
        match self {
            Platform::Any => "",
            Platform::Windows => "__win",
            Platform::WindowsX64 => "__win_x64",
            Platform::WindowsArm64 => "__win_arm",
            Platform::Linux => "__linux",
            Platform::LinuxPacman => "__linux_pacman",
            Platform::LinuxApt => "__linux_apt",
            Platform::Macos => "__macos",
        }
    }
}
impl cu::Parse for Platform {
    type Output = Self;
    fn parse_borrowed(x: &str) -> cu::Result<Self::Output> {
        match x {
            "win" => Ok(Self::Windows),
            "win-x64" => Ok(Self::WindowsX64),
            "win-arm" => Ok(Self::WindowsArm64),
            "linux" => Ok(Self::Linux),
            "linux-pacman" => Ok(Self::LinuxPacman),
            "linux-apt" => Ok(Self::LinuxApt),
            "mac" => Ok(Self::Macos),
            _ => cu::bail!("unknown platform identifier '{x}'"),
        }
    }
}
