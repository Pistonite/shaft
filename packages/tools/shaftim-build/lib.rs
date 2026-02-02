use std::collections::BTreeMap;
#[cfg(feature = "build")]
use std::path::Path;

use cu::pre::*;

pub type ShimConfig = BTreeMap<String, ShimCommand>;

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

#[cfg(feature = "build")]
pub fn build(config_path: &Path) -> cu::Result<String> {
    println!("cargo::rerun-if-changed={}", config_path.as_utf8()?);
    match build_internal(config_path) {
        Ok(tokens) => Ok(tokens.to_string()),
        Err(e) => {
            for line in format!("{e:?}").lines() {
                println!("cargo::error={line}");
            }
            cu::bail!("shim build failed");
        }
    }
}
#[cfg(feature = "build")]
fn build_internal(config_path: &Path) -> cu::Result<pm::TokenStream2> {
    let config = json::parse::<ShimConfig>(&cu::fs::read_string(config_path)?)?;
    if config.is_empty() {
        return Ok(pm::quote! { fn main() {} });
    }

    let cap = config.len();
    let mut match_patterns = Vec::with_capacity(cap);
    let mut match_blocks = Vec::with_capacity(cap);

    let mut max_exe_bytes = 0;

    for (exe_name, command) in config {
        let exe_name = fix_exe_name(&exe_name)?;
        max_exe_bytes = max_exe_bytes.max(exe_name.len());
        let pattern = pm::Literal2::byte_string(exe_name.as_bytes());
        match_patterns.push(pattern);
        if command.bash {
            if cfg!(not(windows)) {
                cu::bail!("for {exe_name}: bash=true may only be specified on Windows");
            }

            let set_path_impl = if command.paths.is_empty() {
                pm::quote! { None }
            } else {
                let sep = if cfg!(windows) { ";" } else { ":" };
                let path_prefix = command.paths.join(sep);
                pm::quote! { Some(#path_prefix) }
            };
            let target = command.target;
            let args = command.args;
            match_blocks.push(pm::quote! {
                return lib::exec_bash_replace(&[ #target, #( ,#args )* ], args, #set_path_impl);
            });
            continue;
        }

        let set_path_impl = if !command.paths.is_empty() {
            let sep = if cfg!(windows) { ";" } else { ":" };
            let path_prefix = command.paths.join(sep);
            pm::quote! { lib::set_path(&mut c, #path_prefix); }
        } else {
            pm::quote! {}
        };

        let set_args_impl = if !command.args.is_empty() {
            let args = command.args;
            pm::quote! { c.args([ #( #args, )* ]); }
        } else {
            pm::quote! {}
        };

        let target = command.target;
        match_blocks.push(pm::quote! {
            #[allow(unused_mut)]
            let mut c = Command::new(#target);
            #set_path_impl
            #set_args_impl
            c
        });
    }

    let output = pm::quote! {
        use shaftim as lib;
        use std::process::{ExitCode, Command};
        fn main() -> ExitCode {
            let mut args = std::env::args_os();
            let Some(arg0) = args.next() else {
                return ExitCode::FAILURE;
            };
            let mut exe_bytes = [0u8; #max_exe_bytes];
            let len = lib::fix_exe_name(&arg0, &mut exe_bytes);
            let mut cmd = match &exe_bytes[..len] {
                #(
                    #match_patterns => { #match_blocks }
                )*
                _ => {
                    eprintln!("shaft-shim: (2) invalid executable: {}", arg0.display());
                    return ExitCode::FAILURE;
                }
            };
            cmd.args(args);
            lib::exec_replace(cmd)
        }
    };

    Ok(output)
}

#[cfg(feature = "build")]
fn fix_exe_name(s: &str) -> cu::Result<String> {
    // we want:
    // - no .cmd or .exe
    // - lowercase
    //
    // this is needed for matching
    let mut lower = s.to_lowercase();
    if let Some(s) = lower.strip_suffix(".cmd") {
        lower = s.to_string();
    } else if let Some(s) = lower.strip_suffix(".exe") {
        lower = s.to_string();
    }
    if lower.contains(['/', '\\']) {
        cu::bail!("invalid executable name: {lower} (no slashes allowed)");
    }
    Ok(lower)
}
