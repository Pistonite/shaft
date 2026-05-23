use std::collections::BTreeMap;

use cu::pre::*;

pub type ShimConfig = BTreeMap<String, ShimCommand>;
#[cfg(feature = "build")]
mod lib_build;
#[cfg(feature = "build")]
pub use lib_build::build;

/// Command configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShimCommand {
    /// The target binary
    target: String,
    /// The extra arguments to pass to target binary (additional CLI args follow the last arg specified, no extra -- in
    /// between)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    args: Vec<String>,
    /// Only effective on Windows. The command will be wrapped with bash.exe
    #[serde(default)]
    #[serde(skip_serializing_if = "bool_is_false")]
    bash: bool,
    /// Additional PATHs to prepend before executing. This is useful
    /// to optimize hotspot executables where only the first invocation
    /// would be through the shim, and subprocesses would invoke the real executable
    /// directly.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    paths: Vec<String>,
}

fn bool_is_false(b: &bool) -> bool {
    !b
}

impl ShimCommand {
    /// Create a shim to the target executable, without any args
    #[inline(always)]
    pub fn target(target: impl Into<String>) -> Self {
        Self {
            target: target.into(),
            args: Default::default(),
            bash: false,
            paths: Default::default(),
        }
    }
    /// Set additional args
    #[inline(always)]
    pub fn args(mut self, args: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.args = args.into_iter().map(|x| x.into()).collect();
        self
    }
    /// Set the target to be wrapped with bash
    #[inline(always)]
    pub fn bash(mut self) -> Self {
        self.bash = true;
        self
    }
    /// Set additional PATH to prepend before executing
    #[inline(always)]
    pub fn paths(mut self, paths: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.paths = paths.into_iter().map(|x| x.into()).collect();
        self
    }
}
