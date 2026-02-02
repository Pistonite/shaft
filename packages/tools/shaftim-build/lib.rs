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
    ///
    /// This can only be used if there are no additional args
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
    /// Create a shim to run the target executable with extra args
    #[inline(always)]
    pub fn target_args(
        target: impl Into<String>,
        args: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        Self {
            target: target.into(),
            args: args.into_iter().map(|x| x.into()).collect(),
            bash: false,
            paths: Default::default(),
        }
    }
    /// Create a shim to run target executable in bash, without any args
    #[inline(always)]
    pub fn target_bash(target: impl Into<String>) -> Self {
        Self {
            target: target.into(),
            args: Default::default(),
            bash: true,
            paths: Default::default(),
        }
    }
    /// Create a shim to run target executable in bash with extra args
    #[inline(always)]
    pub fn target_bash_args(
        target: impl Into<String>,
        args: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        Self {
            target: target.into(),
            args: args.into_iter().map(|x| x.into()).collect(),
            bash: true,
            paths: Default::default(),
        }
    }
    /// Create a shim to prepend multiple paths to PATH, then run the executable.
    /// The order will be the same as paths passed in. The first path in `paths`
    /// will be the first path in the PATH environment variable.
    #[inline(always)]
    pub fn target_paths(
        target: impl Into<String>,
        paths: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        Self {
            target: target.into(),
            args: Default::default(),
            bash: false,
            paths: paths.into_iter().map(|x| x.into()).collect(),
        }
    }
    /// Create a shim to prepend multiple paths to PATH, then run the executable wrapped with bash.
    /// The order will be the same as paths passed in. The first path in `paths`
    /// will be the first path in the PATH environment variable.
    #[inline(always)]
    pub fn target_bash_paths(
        target: impl Into<String>,
        paths: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        Self {
            target: target.into(),
            args: Default::default(),
            bash: true,
            paths: paths.into_iter().map(|x| x.into()).collect(),
        }
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

    for (exe_name, command) in config {
        let pattern = pm::Literal2::byte_string(exe_name.as_bytes());
        match_patterns.push(pattern);
        if command.bash {
            if cfg!(not(windows)) {
                cu::bail!("for {exe_name}: bash=true may only be specified on Windows");
            }

            let set_path_impl = if command.paths.is_empty() {
                pm::quote! { None }
            } else {
                if !command.args.is_empty() {
                    cu::bail!("for {exe_name}: cannot specify both args and paths");
                }
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
            if !command.args.is_empty() {
                cu::bail!("for {exe_name}: cannot specify both args and paths");
            }
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
            let mut cmd = match lib::exe_name(&arg0) {
                #(
                    #match_patterns => { #match_blocks }
                )*
                _ => {
                    eprintln!("shaft-shim: invalid executable");
                    return ExitCode::FAILURE;
                }
            };
            cmd.args(args);
            lib::exec_replace(cmd)
        }
    };

    Ok(output)
}
