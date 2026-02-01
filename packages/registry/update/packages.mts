import type { MetafilePkg, MetaKeyValues } from "./metafile.mts"

import { fetch_from_github_release } from "./fetch/github_release.mts";
import { fetch_from_cratesio } from "./fetch/cratesio.mts";
import { fetch_from_arch_linux } from "./fetch/arch_linux.mts";
import { fetch_from_cargo_toml } from "./fetch/github_cargo_toml.mts";
import { fetch_latest_commit } from "./fetch/latest_commit.mts";
import { fetch_release_name } from "./fetch/release_description.mts";
import { strip_v } from "./util.mts";

const cfg_windows = (x: string) => `'cfg(windows)'.${x}`;
const cfg_windows_arm64 = (x: string) => `'cfg(all(windows,target_arch="aarch64"))'.${x}`;
const cfg_windows_x64 = (x: string) => `'cfg(all(windows,target_arch="x86_64"))'.${x}`;
const cfg_arm64 = (x: string) => `'cfg(target_arch="aarch64")'.${x}`;
const cfg_x64 = (x: string) => `'cfg(target_arch="x86_64")'.${x}`;
const cfg_linux = (x: string) => `'cfg(target_os="linux")'.${x}`;

const default_cratesio_fetcher = (crate: string): PackageFn => {
    return () => fetch_from_cratesio({crate});
};

export type PackageFn = (meta: MetafilePkg) => Promise<MetaKeyValues> | Promise<MetaKeyValues>[] | Promise<Promise<MetaKeyValues>[]>;
export const pkg__7z: PackageFn = (meta) => 
    fetch_from_github_release({
        repo: meta.repo(),
        artifacts: (tag) => {
            const version_no_dot = tag.replace(".","");
            return [`7z${version_no_dot}-arm64.exe`, `7z${version_no_dot}-x64.exe`];
        },
        query: (_, tag, [arm, x64]) => ({
            VERSION: tag,
            [cfg_arm64("SHA")]: arm.sha,
            [cfg_x64("SHA")]: x64.sha,
        })
    });
export const pkg_pwsh: PackageFn = (meta) =>
    fetch_from_github_release({
        repo: meta.repo(),
        tag: (tags) => {
            for (const tag of tags) { if (tag.includes("preview")) { return tag; } }
            throw new Error("failed to find pwsh preview release");
        },
        artifacts: (tag) => {
            tag = strip_v(tag);
            return [`PowerShell-${tag}-win-arm64.zip`, `PowerShell-${tag}-win-x64.zip`];
        },
        query: (_, tag, [arm, x64]) => ({
            VERSION: strip_v(tag),
            [cfg_arm64("SHA")]: arm.sha,
            [cfg_x64("SHA")]: x64.sha,
        })
    });
export const pkg_cargo_binstall = default_cratesio_fetcher("cargo-binstall");
export const pkg_coreutils: PackageFn = () => [
    fetch_from_cratesio({ crate: "eza", query: (v) => ({ "eza.VERSION": v }) }),
    fetch_from_cratesio({ crate: "coreutils", query: (v) => ({ "uutils.VERSION": v }) }),
    fetch_from_arch_linux({ package: "zip", query: (version) => ({ "zip.VERSION": version }) }),
    fetch_from_arch_linux({ package: "unzip", query: (version) => ({ "unzip.VERSION": version }) }),
    fetch_from_arch_linux({ package: "tar", query: (version) => ({ "tar.VERSION": version }) }),
    fetch_from_arch_linux({ package: "which", query: (version) => ({ "which.VERSION": version }) }),
    fetch_from_arch_linux({ package: "bash", query: (version) => ({ "bash.VERSION": version }) }),
    fetch_from_arch_linux({ package: "bash-completion", query: (version) => ({ "bash_cmp.VERSION": version }) }),
];
export const pkg_shellutils: PackageFn = async (meta) => {
    const repo = meta.repo();
    const commit = await fetch_latest_commit(repo);
    return [
        Promise.resolve({ REPO: repo, COMMIT: commit, }),
        fetch_from_cargo_toml({
            repo, ref: commit, path: "packages/which/Cargo.toml",
            query: (v) => ({ "which.VERSION": v })
        }),
        fetch_from_cargo_toml({
            repo, ref: commit, path: "packages/viopen/Cargo.toml",
            query: (v) => ({ "viopen.VERSION": v })
        }),
        fetch_from_cargo_toml({
            repo, ref: commit, path: "packages/vipath/Cargo.toml",
            query: (v) => ({ "vipath.VERSION": v })
        }),
        fetch_from_cargo_toml({
            repo, ref: commit, path: "packages/n/Cargo.toml",
            query: (v) => ({ "n.VERSION": v })
        }),
        fetch_from_cargo_toml({
            repo, ref: commit, path: "packages/wsclip/Cargo.toml",
            query: (v) => ({ "wsclip.VERSION": v })
        }),
    ];
}
export const pkg_git: PackageFn = async () => [
    fetch_from_github_release({
        repo: "https://github.com/microsoft/git",
        query: (_, tag) => {
            tag = strip_v(tag);
            const i = tag.indexOf(".vfs");
            if (i===-1) { throw new Error("latest microsoft/git is not vfs"); }
            return { [cfg_windows("VERSION")]: tag.substring(0, i) }
        }
    }),
    fetch_from_arch_linux({ package: "git", query: (version) => ({ [cfg_linux("VERSION")]: version }) }),
    fetch_from_cratesio({ crate: "git-delta", query: (version) => ({ "delta.VERSION": version }) }),
]
export const pkg_perl: PackageFn = () => fetch_from_arch_linux({ package: "perl", query: (v) => ({ [cfg_linux("VERSION")]: v }) });
export const pkg_curl: PackageFn = () => fetch_from_arch_linux({ package: "curl", query: (v) => ({ [cfg_linux("VERSION")]: v }) });
export const pkg_wget: PackageFn = (meta) => [
    fetch_from_arch_linux({ package: "wget", query: (version) => ({ [cfg_linux("VERSION")]: version }) }),
    fetch_from_github_release({
        repo: meta.get(cfg_windows("REPO")),
        artifact: (all) => {
            const artifact = all.find(x => x.startsWith("wget-ucrt64-xp-openssl-lite"));
            if (!artifact) { throw new Error("could not find wget artifact for windows"); }
            return [ artifact ];
        },
        query: (_, tag, [art]) => {
            const vparts = tag.split('.');
            if (vparts.length < 3) { throw new Error("invalid wget tag format: "+tag); }
            const version = strip_v(vparts[0] + "." + vparts[1] + "." + vparts[2]);
            return {
                [cfg_windows("VERSION")]: version,
                [cfg_windows("URL")]: art.url,
                [cfg_windows("SHA")]: art.sha,
            };
        }
    })
];
export const pkg_fzf: PackageFn = (meta) =>
    fetch_from_github_release({
        repo: meta.repo(),
        artifacts: (tag) => {
            tag = strip_v(tag);
            return [`fzf-${tag}-windows_arm64.zip`, `fzf-${tag}-windows_amd64.zip`];
        },
        query: (_, tag, [arm64, x64]) => ({
            VERSION: strip_v(tag),
            [cfg_windows_arm64("SHA")]: arm64.sha,
            [cfg_windows_x64("SHA")]: x64.sha,
        })
    });
export const pkg_jq: PackageFn = (meta) =>
    fetch_from_github_release({
        repo: meta.repo(),
        artifacts: () => ["jq-windows-amd64.exe"],
        query: (_, tag, [artifact]) => {
            if (!tag.startsWith("jq-")) { throw new Error("invalid jq tag format: "+tag); }
            return {
                VERSION: tag.substring(3),
                [cfg_windows("SHA")]: artifact.sha,
            };
        }
    });
export const pkg_task: PackageFn = (meta) =>
    fetch_from_github_release({
        repo: meta.repo(),
        artifacts: () => ["task_windows_arm64.zip", "task_windows_amd64.zip", "task_linux_amd64.tar.gz"],
        query: (_, tag, [arm64, x64, linux]) => ({
            VERSION: strip_v(tag),
            [cfg_windows_arm64("SHA")]: arm64.sha,
            [cfg_windows_x64("SHA")]: x64.sha,
            [cfg_linux("SHA")]: linux.sha,
        })
    });
export const pkg_bat = default_cratesio_fetcher("bat");
export const pkg_dust = default_cratesio_fetcher("du-dust");
export const pkg_fd = default_cratesio_fetcher("fd-find");
export const pkg_websocat = default_cratesio_fetcher("websocat");
export const pkg_zoxide = default_cratesio_fetcher("zoxide");
export const pkg_hack_font: PackageFn = (meta) => [
    fetch_from_arch_linux({
        package: "ttf-hack-nerd",
        query: (ver, rel) => ({ VERSION_PACMAN: `${ver}-${rel}` })
    }),
    fetch_from_github_release({
        repo: meta.repo(),
        artifacts: () => ["Hack.zip"],
        query: (_, tag, [artifact]) => ({
            VERSION: strip_v(tag),
            SHA: artifact.sha,
        })
    })
];
export const pkg_volta: PackageFn = (meta) =>
    fetch_from_github_release({
        repo: meta.get("pnpm.REPO"),
        query: (_, tag) => ({ "pnpm.VERSION": strip_v(tag) })
    });
export const pkg_uv = default_cratesio_fetcher("uv");
export const pkg_clang: PackageFn = (meta) => [
    fetch_from_github_release({
        repo: meta.repo(),
        artifacts: (tag) => [
            `llvm-mingw-${tag}-ucrt-aarch64.zip`,
            `llvm-mingw-${tag}-ucrt-x86_64.zip`,
        ],
        query: async (repo, tag, [arm, x64]) => {
            const body = (await fetch_release_name(repo, tag)).trim();
            const match = body.match(/LLVM (\d+\.\d+\.\d+)/);
            if (!match) { throw new Error(`failed to find LLVM version in release description`); }
            const version = match[1];
            console.log("-- -- llvm-mingw version is "+version);
            return {
                "TAG": tag,
                [cfg_windows("LLVM_VERSION")]: version,
                [cfg_windows_arm64("SHA")]: arm.sha,
                [cfg_windows_x64("SHA")]: x64.sha,
            };
        }
    }),
    fetch_from_arch_linux({ package: "clang", query: (v) => ({ [cfg_linux("LLVM_VERSION")]: v }) })
];
export const pkg_cmake: PackageFn = (meta) => [
    fetch_from_github_release({
        repo: meta.repo(),
        artifacts: (tag) => {
            tag = strip_v(tag);
            return [`cmake-${tag}-windows-arm64.zip`, `cmake-${tag}-windows-x86_64.zip` ]
        },
        query: async (_, tag, [arm, x64]) => ({
            [cfg_windows("VERSION")]: strip_v(tag),
            [cfg_windows_arm64("SHA")]: arm.sha,
            [cfg_windows_x64("SHA")]: x64.sha,
        })
    }),
    fetch_from_arch_linux({ package: "cmake", query: (v) => ({ [cfg_linux("VERSION")]: v }) })
];
