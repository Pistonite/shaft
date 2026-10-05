---
name: add-github-release-package
description: Add a new shaft registry package that installs a tool by downloading a prebuilt archive/binary from a GitHub release (e.g. gh, bun, nvim, tree-sitter). Use when asked to "add a package", "add support for <tool>" or "install <tool> from github release" in the shaft repo.
---

# Add a package downloaded from a GitHub release

The steps below cover a registry package that pins a release version and
SHA256s in `metadata.toml`, downloads the asset, extracts it into the install
dir, and links the binary. Existing examples to copy from:

| Example                                         | Asset shape                                   |
|-------------------------------------------------|-----------------------------------------------|
| `packages/registry/src/packages/bun.rs`         | zip with a top-level dir -> `unarchive_rename`  |
| `packages/registry/src/packages/nvim/mod.rs`    | zip / tar.gz with a top-level dir, `bin/` inside |
| `packages/registry/src/packages/shellutils/task.rs` | flat archive -> `unarchive` to temp + copy the binary |
| `packages/registry/src/packages/tree-sitter.rs` | single gzipped binary -> `opfs::ungz_bytes`     |
| `packages/registry/src/packages/7z/mod_mac.rs`  | platform-specific file (`mod_<target>.rs`)    |

## Hard rules

- **No manual testing.** Do not run `shaft sync`, `x shaft ...` or the
  installed tool, and do not install anything on the machine. Verification
  means only regenerating the code and running the checks (step 6).
- **No git writes.** Do not `git commit`, `git push`, create branches or
  open PRs. Leave all changes uncommitted for the user to review.
- Do not hand-write SHA256s or versions. Let the update script fetch them.

## Background: how the registry works

- Each file `packages/registry/src/packages/<name>.rs` (or
  `<name>/mod.rs`, `<name>/mod_<target>.rs`) is one package. The name must
  be **kebab-case** and becomes `PkgId` (`foo-bar` -> `PkgId::FooBar`). The
  **first line of the `//!` doc comment** is the required short description.
- Target suffixes (`packages/build/src/build_registry/packages/platform.rs`)
  are `_win`, `_win_x64`, `_win_arm`, `_linux`, `_linux_x64`,
  `_linux-pacman`, `_linux-apt`, `_mac` and `_mac_arm`. The only platforms
  that exist are **Windows x64/arm64, Linux x64 and macOS arm64**, so a file
  with no suffix covers exactly these four and no extra arch gating is
  needed.
- `packages/registry/metadata.toml` holds a `[snake_name]` table, generated
  into `metadata::snake_name::KEY` (`metadata.gen.rs`). A key ending in
  `.__match__ = "opfs::cpu_arch()"` with `'opfs::CpuArch::X64'` and
  `'opfs::CpuArch::Arm64'` entries becomes a function, for example
  `metadata::foo::SHA()`. Keys prefixed with `'cfg(...)'` are cfg-gated. The
  package `foo-bar` uses table `foo_bar`. A name starting with a digit gets a
  `_` prefix (`_7z`).
- `packages/registry/update/packages.mts` exports `pkg_<table>` per table.
  `fetch_from_github_release` picks the newest release that is at least 14
  days old, sha256s each listed asset, and `query` returns the keys to write.

## Steps

### 1. Inspect the upstream release

Use the GitHub API (`gh api repos/<owner>/<repo>/releases/latest` or
`curl https://api.github.com/repos/<owner>/<repo>/releases/latest`) to find:

- the tag format (`v1.2.3`, `1.2.3`, `foo-v1.2.3`, ...)
- the asset names for **win x64, win arm64, linux x64 (glibc/gnu), mac arm64**
  (watch for `amd64`/`x86_64`/`x64`, `arm64`/`aarch64`, `darwin`/`macOS`/`macos`)
- the **archive layout**: download each asset into the scratchpad and list it
  (`tar tf`, `unzip -l`). Note whether it has a top-level directory and where
  the binary sits (root, `bin/`, ...). Layouts often differ between the
  Windows zip and the unix tarball.
- how `<tool> --version` formats its output (from docs or source), for
  `verify`.

If a platform has no asset, use `"<unsupported>"` for that SHA entry. Only
skip a platform's code path if the user agrees.

### 2. `packages/registry/metadata.toml`

Add the table near related tools:

```toml
[foo_bar]
REPO = "https://github.com/<owner>/<repo>"
VERSION = "0"
'cfg(windows)'.SHA.__match__ = "opfs::cpu_arch()"
'cfg(windows)'.SHA.'opfs::CpuArch::Arm64' = ""
'cfg(windows)'.SHA.'opfs::CpuArch::X64' = ""
'cfg(target_os="linux")'.SHA.__match__ = "opfs::cpu_arch()"
'cfg(target_os="linux")'.SHA.'opfs::CpuArch::Arm64' = "<unsupported>"
'cfg(target_os="linux")'.SHA.'opfs::CpuArch::X64' = ""
'cfg(target_os="macos")'.SHA.__match__ = "opfs::cpu_arch()"
'cfg(target_os="macos")'.SHA.'opfs::CpuArch::Arm64' = ""
'cfg(target_os="macos")'.SHA.'opfs::CpuArch::X64' = "<unsupported>"
```

The placeholder values are overwritten in step 4.

### 3. `packages/registry/update/packages.mts`

Add the fetcher. Use `artifacts: (tag) => [...]` when asset names contain the
version:

```ts
export const pkg_foo_bar: PackageFn = (meta) =>
    fetch_from_github_release({
        repo: meta.repo(),
        artifacts: (tag) => {
            const v = strip_v(tag);
            return [
                `foo_${v}_windows_arm64.zip`,
                `foo_${v}_windows_amd64.zip`,
                `foo_${v}_linux_amd64.tar.gz`,
                `foo_${v}_macOS_arm64.zip`,
            ];
        },
        query: (_, tag, [arm64, x64, linux, mac]) => ({
            VERSION: strip_v(tag),
            ...match_cpu_arch(cfg_windows("SHA"), { arm: arm64.sha, x64: x64.sha }),
            ...match_cpu_arch(cfg_linux("SHA"), { arm: "<unsupported>", x64: linux.sha }),
            ...match_cpu_arch(cfg_macos("SHA"), { arm: mac.sha, x64: "<unsupported>" }),
        }),
    });
```

The order of `artifacts` must match the destructuring in `query`. For an
unusual tag, use `strip(tag, "prefix")`. Pass a `tag:` picker to skip
prereleases (see `pkg_pwsh`).

### 4. Fetch the version and SHAs

```sh
cd packages/registry && node update/main.mts foo_bar
```

Check that `metadata.toml` now has a real `VERSION` and 64-hex SHAs. If the
latest release is newer than 14 days, the script picks an older one; that is
expected.

### 5. `packages/registry/src/packages/foo-bar.rs`

Skeleton (adapt the install step to the layout from step 1):

```rust
//! Foo Bar - one line short description
use crate::pre::*;

register_binaries!("foo");
binary_dependencies!(_7z); // only if any platform extracts a .zip / .7z (tar formats use system tar)

pub fn verify(_: &Context) -> cu::Result<Verified> {
    check_in_shaft!("foo");
    let stdout = command_output!("foo", ["--version"]);
    let Some(version) = /* parse stdout */ else {
        cu::warn!("foo --version returned unexpected output: {stdout}");
        return Ok(Verified::NotUpToDate);
    };
    check_outdated!(version, metadata[foo_bar]::VERSION);
    Ok(Verified::UpToDate)
}

pub fn download(ctx: &Context) -> cu::Result<()> {
    hmgr::download_file(file_name()?, url()?, metadata::foo_bar::SHA(), ctx.bar())?;
    Ok(())
}

pub fn install(ctx: &Context) -> cu::Result<()> {
    opfs::ensure_terminated(bin_name!("foo"))?;
    let archive_path = hmgr::paths::download(file_name()?, url()?);
    let temp_dir = hmgr::paths::temp_dir("foo-bar-extract");
    ctx.move_install_to_old_if_exists()?;
    // top-level dir in archive:
    opfs::unarchive_rename(archive_path, &temp_dir, temp_dir.join(stem()?), ctx.install_dir(), true, None)?;
    // flat archive: pass `&temp_dir` as `from` instead
    Ok(())
}

pub fn configure(ctx: &Context) -> cu::Result<()> {
    let exe = cu::path!((ctx.install_dir()) / "bin" / bin_name!("foo")).into_utf8()?;
    ctx.add_item(Item::link_bin(hmgr::paths::binary(bin_name!("foo")).into_utf8()?, exe))?;
    Ok(())
}

pub fn uninstall(ctx: &Context) -> cu::Result<()> {
    opfs::ensure_terminated(bin_name!("foo"))?;
    ctx.move_install_to_old_if_exists()?;
    Ok(())
}

// fn stem() / file_name() / url(): pick the asset with
// cfg!(windows) + opfs::is_arm(), cfg!(target_os = "linux"), cfg!(target_os = "macos"),
// else cu::bail!("foo not supported on this platform").
// URL: format!("{repo}/releases/download/<tag>/{file_name}") — rebuild the tag
// exactly as upstream formats it (e.g. `v{version}`).
```

Notes:
- `hmgr::download_file` and `hmgr::paths::download` must get the **same**
  identifier and URL.
- `bin_name!` adds `.exe` on Windows.
- Shell completions or aliases go through `Item::bash`, `Item::zsh` and
  `Item::pwsh` (see `shellutils/task.rs`). Add them only if the tool supports
  completions and the user wants them.
- Match the formatting of the surrounding files. Step 6 runs rustfmt.

### 6. Regenerate and check (the only verification)

Run these from the repo root:

```sh
x run-build   # regenerates packages.gen.rs / metadata.gen.rs
x check       # clippy, rustfmt, lfmt, version check
x test        # registry tests (ids in sync, description present)
```

Fix any failures and rerun. If formatting fails, `x fix` may be used. Do
**not** run the package or any `shaft` command.

### 7. Finish

- Bump the patch version in `/VERSION` (for example `0.4.56` -> `0.4.57`)
  unless it has already been bumped in this uncommitted change. Then rerun
  `x check`, since it includes the version check.
- Report the changed files, the pinned version, and the archive layout you
  assumed for each platform.
- Stop. Do not commit or push.
