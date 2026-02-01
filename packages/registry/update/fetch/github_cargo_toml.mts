import type { MetaKeyValues } from "../metafile.mts";
import { from_toml_string, parse_github_repo } from "../util.mts";


export type GitHubCargoTomlArgs = {
    repo: string;
    ref: string;
    path: string;
    query?: (version: string) => Promise<MetaKeyValues> | MetaKeyValues;
};
/** Fetch package version from a Cargo.toml file on GitHub */
export const fetch_from_cargo_toml = async ({ repo, ref, path, query }: GitHubCargoTomlArgs): Promise<MetaKeyValues> => {
    query = query || ((VERSION) => ({ VERSION }));
    console.log(`-- fetching Cargo.toml from ${repo} @ ${ref}: ${path}`);
    const repo_path = parse_github_repo(repo);

    // fetch raw file content
    const raw_url = `https://raw.githubusercontent.com/${repo_path}/${ref}/${path}`;
    const response = await fetch(raw_url);
    if (!response.ok) {
        throw new Error(`failed to fetch ${raw_url}: ${response.status}`);
    }
    const content = await response.text();

    // parse package.version from Cargo.toml
    const version = parse_cargo_version(content);
    console.log(`-- -- parsed version from ${path}: ${version}`);

    return await query(version);
};

/** Parse the package.version from Cargo.toml content */
const parse_cargo_version = (content: string): string => {
    const lines = content.split("\n");
    let in_package_section = false;

    for (const line of lines) {
        const trimmed = line.trim();

        // check for section headers
        if (trimmed.startsWith("[")) {
            in_package_section = trimmed === "[package]";
            continue;
        }

        // look for version key in [package] section
        if (in_package_section && trimmed.startsWith("version")) {
            const match = trimmed.match(/^version\s*=\s*(.+)$/);
            if (match) {
                return from_toml_string(match[1].trim());
            }
        }
    }

    throw new Error("failed to find package.version in Cargo.toml");
};

