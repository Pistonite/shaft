/**
 * Update metadata.toml
 *
 * Usage: node --experimental-strip-types update-metadata.ts [PACKAGE]
 *
 * PACKAGE is the package to update, or all packages if omitted
 */

//* To get type checking in your IDE, install @types/node with a package manager */
/// <reference types="node" />
import { Metafile, type MetaKeyValues } from "./metafile.mts";
import { METADATA_TOML, strip_v, sha256_from_url, github_release_url } from "./util.mts";
import { fetch_from_github_release } from "./fetch/github_release.mts";
import { fetch_from_cratesio } from "./fetch/cratesio.mts";
import { fetch_from_arch_linux } from "./fetch/arch_linux.mts";
import { fetch_from_cargo_toml } from "./fetch/github_cargo_toml.mts";
import { fetch_latest_commit } from "./fetch/latest_commit.mts";


// === CONSTANT HELPERS ======================================================
const cfg_windows = (x: string) => `'cfg(windows)'.${x}`;
const cfg_windows_arm64 = (x: string) => `'cfg(all(windows,target_arch="aarch64"))'.${x}`;
const cfg_windows_x64 = (x: string) => `'cfg(all(windows,target_arch="x86_64"))'.${x}`;
const cfg_arm64 = (x: string) => `'cfg(target_arch="aarch64")'.${x}`;
const cfg_x64 = (x: string) => `'cfg(target_arch="x86_64")'.${x}`;
const cfg_linux = (x: string) => `'cfg(target_os="linux")'.${x}`;

// === CLI ===================================================================
const main = async () => {
    try {
        await main_internal();
    } catch (e) {
        console.error(`ERROR: ${e instanceof Error ? e.message : e}`);
        process.exit(1);
    }
};

const main_internal = async () => {
    const args = process.argv.slice(2);

    if (args.includes("-h") || args.includes("--help")) {
        console.log("Usage: node --experimental-strip-types update-metadata.ts [PACKAGE]");
        console.log();
        console.log("Update metadata.toml with latest package versions.");
        console.log();
        console.log("Arguments:");
        console.log("  PACKAGE  Package to update (updates all if omitted)");
        return;
    }

    const meta = new Metafile(METADATA_TOML);
    meta.open();
    const all_packages = meta.pkg_names();

    // determine which packages to update
    let packages_to_update: string[];
    if (args.length > 0) {
        const pkg = args[0];
        if (!all_packages.includes(pkg)) {
            throw new Error(`package '${pkg}' not found in metadata.toml`);
        }
        packages_to_update = [pkg];
    } else {
        packages_to_update = all_packages;
    }

    const results = await Promise.allSettled(
        packages_to_update.map(pkg => fetch_by_package(meta, pkg))
    );

    let has_error = false;
    const updates: UpdatePayload[] = [];
    for (let i = 0; i < results.length; i++) {
        const pkg = packages_to_update[i];
        const result = results[i];
        if (result.status === "rejected") {
            console.error(`ERROR: ${pkg}: ${result.reason}`);
            has_error = true;
        } else {
            updates.push([ pkg, result.value ]);
        }
    }
    if (has_error) {
        throw new Error("there were errors fetching package updates");
    }

    let updated = false;
    for (const [pkg, key_values] of updates) {
        updated = meta.update(pkg, key_values) || updated;
    }

    if (updated) {
        meta.save();
        console.log("metadata.toml updated");
    } else {
        console.log("already up to date");
    }
};

// === PACKAGES ==============================================================

type UpdatePayload = [string, MetaKeyValues];

const fetch_by_package = async (meta: Metafile, pkg: string): Promise<MetaKeyValues> => {
    console.log(`fetching '${pkg}'...`);
    /** Fetch the latest version of a package, returns an object */
    switch (pkg) {
        case "_7z": {
            return await fetch_from_github_release({
                repo: meta.get(pkg, "REPO"),
                query: async (repo, tag, artifacts) => {
                    const version_no_dot = tag.replace(".","");
                    const arm64_exe = `7z${version_no_dot}-arm64.exe`; if (!artifacts.includes(arm64_exe)) { throw new Error("failed to find 7z arm64 artifact"); }
                    const x64_exe = `7z${version_no_dot}-x64.exe`; if (!artifacts.includes(x64_exe)) { throw new Error("failed to find 7z x64 artifact"); }
                    const arm64_url = github_release_url(repo, tag, arm64_exe);
                    const x64_url = github_release_url(repo, tag, x64_exe);
                    const arm64_hash = await sha256_from_url(arm64_url);
                    const x64_hash = await sha256_from_url(x64_url);

                    return {
                        VERSION: tag,
                        [cfg_arm64("SHA")]: arm64_hash,
                        [cfg_x64("SHA")]: x64_hash,
                    };
                }
            });
        }
        case "pwsh": 
            return await fetch_from_github_release({
                repo: meta.get(pkg, "REPO"),
                tag: (tags) => {
                    for (const tag of tags) { if (tag.includes("preview")) { return tag; } }
                    throw new Error("failed to find pwsh preview release");
                },
                query: async (repo, tag, artifacts) => {
                    tag = strip_v(tag);
                    const arm64_zip = `PowerShell-${tag}-win-arm64.zip`; if (!artifacts.includes(arm64_zip)) { throw new Error("failed to find pwsh arm64 artifact"); }
                    const x64_zip = `PowerShell-${tag}-win-x64.zip`; if (!artifacts.includes(x64_zip)) { throw new Error("failed to find pwsh x64 artifact"); }
                    const arm64_url = github_release_url(repo, 'v'+tag, arm64_zip);
                    const x64_url = github_release_url(repo, 'v'+tag, x64_zip);
                    const arm64_hash = await sha256_from_url(arm64_url);
                    const x64_hash = await sha256_from_url(x64_url);

                    return {
                        VERSION: tag,
                        [cfg_arm64("SHA")]: arm64_hash,
                        [cfg_x64("SHA")]: x64_hash,
                    };
                }
            });
        case "cargo_binstall": return await fetch_from_cratesio({ crate: "cargo-binstall" });
        case "coreutils": {
            return {
                ...await fetch_from_cratesio({ crate: "eza", query: (v) => ({ "eza.VERSION": v }) }),
                ...await fetch_from_cratesio({ crate: "coreutils", query: (v) => ({ "uutils.VERSION": v }) }),
                ...await fetch_from_arch_linux({
                    package: "zip",
                    query: async (version) => { return { "zip.VERSION": version } }
                }),
                ...await fetch_from_arch_linux({
                    package: "unzip",
                    query: async (version) => { return { "unzip.VERSION": version } }
                }),
                ...await fetch_from_arch_linux({
                    package: "tar",
                    query: async (version) => { return { "tar.VERSION": version } }
                }),
                ...await fetch_from_arch_linux({
                    package: "which",
                    query: async (version) => { return { "which.VERSION": version } }
                }),
                ...await fetch_from_arch_linux({
                    package: "bash",
                    query: async (version) => { return { "bash.VERSION": version } }
                }),
                ...await fetch_from_arch_linux({
                    package: "bash-completion",
                    query: async (version) => { return { "bash_cmp.VERSION": version } }
                }),
            };
        }
        case "shellutils": {
            const repo = meta.get(pkg, "REPO");
            const commit = await fetch_latest_commit(repo);
            return {
                REPO: repo,
                COMMIT: commit,
                ...await fetch_from_cargo_toml({
                    repo, ref: commit, path: "packages/which/Cargo.toml",
                    query: (v) => ({ "which.VERSION": v })
                }),
                ...await fetch_from_cargo_toml({
                    repo, ref: commit, path: "packages/viopen/Cargo.toml",
                    query: (v) => ({ "viopen.VERSION": v })
                }),
                ...await fetch_from_cargo_toml({
                    repo, ref: commit, path: "packages/vipath/Cargo.toml",
                    query: (v) => ({ "vipath.VERSION": v })
                }),
                ...await fetch_from_cargo_toml({
                    repo, ref: commit, path: "packages/n/Cargo.toml",
                    query: (v) => ({ "n.VERSION": v })
                }),
                ...await fetch_from_cargo_toml({
                    repo, ref: commit, path: "packages/wsclip/Cargo.toml",
                    query: (v) => ({ "wsclip.VERSION": v })
                }),
            }
        }
        case "git": {
            return {
                ...await fetch_from_github_release({
                    repo: "https://github.com/microsoft/git",
                    query: async (_, tag) => {
                        tag = strip_v(tag);
                        const i = tag.indexOf(".vfs");
                        if (i===-1) { throw new Error("latest microsoft/git is not vfs"); }
                        return { [cfg_windows("VERSION")]: tag.substring(0, i) }
                    }
                }),
                ...await fetch_from_arch_linux({
                    package: "git",
                    query: async (version) => { return { [cfg_linux("VERSION")]: version } }
                }),
                ...await fetch_from_cratesio({
                    crate: "git-delta",
                    query: async (version) => { return { "delta.VERSION": version } }
                }),
            }
        }
        case "perl":
            return await fetch_from_arch_linux({
                package: "perl",
                query: async (version) => { return { [cfg_linux("VERSION")]: version } }
            });
        case "curl":
            return await fetch_from_arch_linux({
                package: "curl",
                query: async (version) => { return { [cfg_linux("VERSION")]: version } }
            });
        case "wget": {
            return {
                ...await fetch_from_arch_linux({
                    package: "wget",
                    query: async (version) => { return { [cfg_linux("VERSION")]: version } }
                }),
                ...await fetch_from_github_release({
                    repo: meta.get(pkg, `'cfg(windows)'.REPO`),
                    query: async (repo, tag, artifacts) => {
                        const artifact = artifacts.find(x => x.startsWith("wget-ucrt64-xp-openssl-lite"));
                        if (!artifact) { throw new Error("could not find wget artifact for windows"); }
                        const vparts = tag.split('.');
                        if (vparts.length < 3) { throw new Error("invalid wget tag format: "+tag); }
                        const version = strip_v(vparts[0] + "." + vparts[1] + "." + vparts[2]);
                        const url = github_release_url(repo, tag, artifact);
                        const sha = await sha256_from_url(url);
                        return {
                            [cfg_windows("VERSION")]: version,
                            [cfg_windows("URL")]: url,
                            [cfg_windows("SHA")]: sha,
                        };
                    }
                })
            };
        }
        case "fzf": {
            return await fetch_from_github_release({
                repo: meta.get(pkg, "REPO"),
                query: async (repo, tag, artifacts) => {
                    tag = strip_v(tag);
                    const arm64_zip = `fzf-${tag}-windows_arm64.zip`; if (!artifacts.includes(arm64_zip)) { throw new Error("failed to find fzf arm64 artifact"); }
                    const x64_zip = `fzf-${tag}-windows_amd64.zip`; if (!artifacts.includes(x64_zip)) { throw new Error("failed to find fzf x64 artifact"); }
                    const arm64_url = github_release_url(repo, 'v'+tag, arm64_zip);
                    const x64_url = github_release_url(repo, 'v'+tag, x64_zip);
                    const arm64_hash = await sha256_from_url(arm64_url);
                    const x64_hash = await sha256_from_url(x64_url);
                    return {
                        VERSION: tag,
                        [cfg_windows_arm64("SHA")]: arm64_hash,
                        [cfg_windows_x64("SHA")]: x64_hash,
                    };
                }
            });
        }
        case "jq": {
            return await fetch_from_github_release({
                repo: meta.get(pkg, "REPO"),
                query: async (repo, tag, artifacts) => {
                    if (!tag.startsWith("jq-")) { throw new Error("invalid jq tag format: "+tag); }
                    const artifact = "jq-windows-amd64.exe"; if (!artifacts.includes(artifact)) { throw new Error("failed to find jq artifact"); }
                    const url = github_release_url(repo, tag, artifact);
                    const sha = await sha256_from_url(url);
                    return {
                        VERSION: tag.substring(3),
                        [cfg_windows("SHA")]: sha,
                    };
                }
            });
        }
        case "task": {
            return await fetch_from_github_release({
                repo: meta.get(pkg, "REPO"),
                query: async (repo, tag, artifacts) => {
                    tag = strip_v(tag);
                    const artifact_arm64 = "task_windows_arm64.zip"; if (!artifacts.includes(artifact_arm64)) { throw new Error("failed to find task arm64 artifact"); }
                    const artifact_x64 = "task_windows_amd64.zip"; if (!artifacts.includes(artifact_x64)) { throw new Error("failed to find task x64 artifact"); }
                    const artifact_linux = "task_linux_amd64.tar.gz"; if (!artifacts.includes(artifact_linux)) { throw new Error("failed to find task linux artifact"); }
                    const url_arm64 = github_release_url(repo, 'v'+tag, artifact_arm64);
                    const url_x64 = github_release_url(repo, 'v'+tag, artifact_x64);
                    const url_linux = github_release_url(repo, 'v'+tag, artifact_linux);
                    const sha_arm64 = await sha256_from_url(url_arm64);
                    const sha_x64 = await sha256_from_url(url_x64);
                    const sha_linux = await sha256_from_url(url_linux);
                    return {
                        VERSION: tag,
                        [cfg_windows_arm64("SHA")]: sha_arm64,
                        [cfg_windows_x64("SHA")]: sha_x64,
                        [cfg_linux("SHA")]: sha_linux,
                    };
                }
            });
        }
        case "bat": return await fetch_from_cratesio({ crate: "bat" });
        case "dust": return await fetch_from_cratesio({ crate: "du-dust" });
        case "fd": return await fetch_from_cratesio({ crate: "fd-find" });
        case "websocat": return await fetch_from_cratesio({ crate: "websocat" });
        case "zoxide": return await fetch_from_cratesio({ crate: "zoxide" });
        case "hack_font": {
            return {
                ...await fetch_from_arch_linux({
                    package: "ttf-hack-nerd",
                    query: (ver,rel) => ({VERSION_PACMAN:`${ver}-${rel}`})
                }),
                ...await fetch_from_github_release({
                    repo: meta.get(pkg, "REPO"),
                    query: async (repo, tag, artifacts) => {
                        tag = strip_v(tag);
                        const artifact = "Hack.zip";
                        if (!artifacts.includes(artifact)) { throw new Error("failed to find Hack.zip"); }
                        const url = github_release_url(repo, 'v'+tag, artifact);
                        const sha = await sha256_from_url(url);
                        return {
                            VERSION: tag,
                            SHA: sha,
                        };
                    }
                }),
            }
        }
        case "volta": {
            return await fetch_from_github_release({
                repo: meta.get(pkg, "pnpm.REPO"),
                query: (_, tag) => ({ "pnpm.VERSION": strip_v(tag) })
            });
        }
        case "uv": return await fetch_from_cratesio({ crate: "uv" });

        default:
            console.log(`WARNING: unknown package '${pkg}'`);
            return {};
    }
};

void main();
