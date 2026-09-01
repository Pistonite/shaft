//! NodeJS and package managers serviced through `pnpm` the performant package manager

use crate::pre::*;

register_binaries!("node", "pnpm");
binary_dependencies!(_7z);
version_cache!(static ALIAS_VERSION = metadata::pnpm::ALIAS_VERSION);

pub fn verify(_: &Context) -> cu::Result<Verified> {
    check_in_shaft!("pnpm");
    let version = (|| cu::Ok(command_output!("pnpm", ["--version"])))();
    let Ok(version) = version else {
        return Ok(Verified::NotInstalled);
    };
    check_in_shaft!("pnpx");
    check_outdated!(version.trim(), metadata[pnpm]::VERSION);
    check_in_path!("node");
    // check_in_path!("yarn");
    check_config_version_cache!(ALIAS_VERSION);
    Ok(Verified::UpToDate)
}

pub fn download(ctx: &Context) -> cu::Result<()> {
    hmgr::download_file(
        pnpm_file_name()?,
        pnpm_url()?,
        metadata::pnpm::SHA(),
        ctx.bar(),
    )?;
    Ok(())
}
pub fn install(ctx: &Context) -> cu::Result<()> {
    opfs::ensure_terminated(bin_name!("node"))?;
    opfs::ensure_terminated(bin_name!("pnpm"))?;
    opfs::ensure_terminated(bin_name!("yarn"))?;
    let install_dir = ctx.install_dir();
    let pnpm_archive = hmgr::paths::download(pnpm_file_name()?, pnpm_url()?);
    // the home only hosts pnpm binary, pnpm home is ~/.pnpm, so we will clean
    // the install directory on install
    opfs::unarchive(&pnpm_archive, &install_dir, true)?;
    Ok(())
}
pub fn uninstall(_: &Context) -> cu::Result<()> {
    if cfg!(windows) {
        if let Ok(local) = cu::env_var("LOCALAPPDATA") {
            cu::fs::rec_remove(Path::new(&local).join("npm-cache"))?;
        }
    } else {
        if let Some(home) = std::env::home_dir() {
            cu::fs::rec_remove(home.join(".npm"))?;
            cu::fs::rec_remove(home.join(".pnpm"))?;
        }
    }
    Ok(())
}
pub fn configure(ctx: &Context) -> cu::Result<()> {
    let home = cu::check!(std::env::home_dir(), "failed to get home directory")?;
    let pnpm_home = cu::path!(home / ".pnpm");
    ctx.add_item(Item::user_env_var("PNPM_HOME", pnpm_home.as_utf8()?))?;
    let pnpm_home_bin = cu::path!(&pnpm_home / "bin");
    ctx.add_item(Item::user_path(pnpm_home_bin.as_utf8()?))?;

    let install_dir = ctx.install_dir();
    // pnpm may reject calls if the bin dir is not in path
    let modified_path = {
        let current_path = cu::env_var("PATH").unwrap_or_default();
        let mut new_path = pnpm_home_bin.as_utf8()?.to_string();
        if !current_path.is_empty() {
            if cfg!(windows) {
                new_path.push(';');
            } else {
                new_path.push(':');
            }
            new_path.push_str(&current_path);
        }
        new_path
    };

    let pnpm_bin = install_dir.join(bin_name!("pnpm"));
    let pnpm_bin_str = pnpm_bin.clone().into_utf8()?;
    // pnpm requires shim because it ships with node-gyp and is discovered
    // from the real binary path
    ctx.add_item(Item::shim_bin(
        bin_name!("pnpm"),
        ShimCommand::target(pnpm_bin_str.clone()),
    ))?;
    ctx.add_item(Item::shim_bin(
        bin_name!("pnpx"),
        ShimCommand::target(pnpm_bin_str).args(["dlx"]),
    ))?;
    // note we don't expose pn/pnx alias because I don't use them

    let config = ctx.load_config(CONFIG)?;
    // configure registry first
    let registry = if config.global_registry.is_empty() {
        cu::print!(
            "using default registry: {}",
            metadata::pnpm::DEFAULT_REGISTRY
        );
        metadata::pnpm::DEFAULT_REGISTRY
    } else {
        cu::warn!("pinning global pnpm registry to {}", config.global_registry);
        if !config.install_npm.is_empty() {
            cu::warn!("pinning global npm registry to {}", config.global_registry);
        }
        &config.global_registry
    };
    {
        let result = pnpm_bin
            .command()
            .args(["config", "set", "--global", registry])
            .env("PNPM_HOME", &pnpm_home)
            .env("PATH", &modified_path)
            .stderr(cu::lv::E)
            .stdio_null()
            .wait_nz();
        cu::check!(result, "failed to set pnpm registry")?;
    }

    let default_version = &config.default_version;
    {
        let mut package = "node".to_string();
        let version = &default_version.node;
        let resolved_version = if version.is_empty() {
            metadata::pnpm::node::DEFAULT_VERSION
        } else {
            cu::warn!("node version is pinned to {version}");
            version
        };
        package.push('@');
        package.push_str(resolved_version);
        let (child, bar, _) = pnpm_bin
            .command()
            .args(["install", "--global", &package])
            .env("PNPM_HOME", &pnpm_home)
            .env("PATH", &modified_path)
            .stdoe(
                cu::pio::spinner(format!("pnpm install {package}"))
                    .configure_spinner(|b| b.parent(ctx.bar())),
            )
            .stdin_null()
            .spawn()?;
        child.wait_nz()?;
        bar.done();
    }

    if !config.install_npm.is_empty() {
        let mut package = "npm".to_string();
        let version = &config.install_npm;
        let resolved_version = if version == "*" {
            metadata::pnpm::npm::DEFAULT_VERSION
        } else {
            cu::warn!("npm version is pinned to {version}");
            version
        };
        package.push('@');
        package.push_str(resolved_version);
        let (child, bar, _) = pnpm_bin
            .command()
            .args(["install", "--global", &package])
            .env("PNPM_HOME", &pnpm_home)
            .env("PATH", &modified_path)
            .stdoe(
                cu::pio::spinner(format!("npm install {package}"))
                    .configure_spinner(|b| b.parent(ctx.bar())),
            )
            .stdin_null()
            .spawn()?;
        child.wait_nz()?;
        bar.done();

        let npm_bin = pnpm_home_bin.join(bin_name!("npm.exe"));
        let result = npm_bin
            .command()
            .args(["config", "set", "--global", registry])
            .env("PNPM_HOME", &pnpm_home)
            .env("PATH", &modified_path)
            .stderr(cu::lv::E)
            .stdio_null()
            .wait_nz();
        cu::check!(result, "failed to set npm registry")?;
    }

    ALIAS_VERSION.update()?;
    Ok(())
}

pub fn clean(ctx: &Context) -> cu::Result<()> {
    if let Ok(pnpm) = cu::which("pnpm") {
        let (child, bar, _) = pnpm
            .command()
            .args(["store", "prune"])
            .stdoe(
                cu::pio::spinner("cleaning pnpm store")
                    .configure_spinner(|builder| builder.parent(ctx.bar())),
            )
            .stdin_null()
            .spawn()?;
        child.wait_nz()?;
        bar.done();
    }
    Ok(())
}

fn pnpm_url() -> cu::Result<String> {
    let version = metadata::pnpm::VERSION;
    let artifact = pnpm_file_name()?;
    let repo = metadata::pnpm::REPO;
    Ok(format!("{repo}/releases/download/v{version}/{artifact}"))
}

fn pnpm_file_name() -> cu::Result<&'static str> {
    if cfg!(windows) {
        if opfs::is_arm() {
            Ok("pnpm-win32-arm64.zip")
        } else {
            Ok("pnpm-win32-x64.zip")
        }
    } else if cfg!(target_os = "linux") {
        Ok("pnpm-linux-x64.tar.gz")
    } else if cfg!(target_os = "macos") {
        Ok("pnpm-darwin-x64.tar.gz")
    } else {
        cu::bail!("platform not supported");
    }
}

config_file! {
    static CONFIG: Config = {
        template: include_str!("config.toml"),
        migration: [
            include_str!("migrate_v0.js"),
            include_str!("migrate_v1.js"),
            include_str!("migrate_v2.js"),
        ],
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
struct Config {
    #[serde(default)]
    pub default_version: ConfigDefaultVersion,
    #[serde(default)]
    pub global_registry: String,
    #[serde(default)]
    pub install_npm: String,
}
#[derive(Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct ConfigDefaultVersion {
    #[serde(default)]
    pub node: String,
}
