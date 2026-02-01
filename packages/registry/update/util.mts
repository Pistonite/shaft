/// <reference types="node" />
import * as fs from "node:fs";
import * as path from "node:path";
import { fileURLToPath } from "node:url";
import { execSync } from "node:child_process";

export const GITHUB_API = "https://api.github.com/";
export const CRATESIO_API = "https://crates.io/api/v1/";
export const ARCHLINUX_API = "https://archlinux.org/packages/search/json/";
const SCRIPT_DIR = path.dirname(fileURLToPath(import.meta.url));
const TEMP_DIR = path.join(SCRIPT_DIR, "temp");
export const METADATA_TOML = path.join(path.dirname(SCRIPT_DIR), "metadata.toml");

/** Extract owner/repo path from a GitHub URL */
export const parse_github_repo = (repo: string): string => {
    const match = repo.match(/github\.com\/([^/]+\/[^/]+)/);
    if (!match) {
        throw new Error(`invalid github repo url: ${repo}`);
    }
    return match[1].replace(/\.git$/, "");
};

/** Strip leading 'v' from a version tag */
export const strip_v = (version: string): string => {
    return version[0]==="v" ? version.substring(1) : version;
};

/** Download a file and compute its SHA256 hash */
export const sha256_from_url = async (url: string): Promise<string> => {
    console.log(`-- hashing ${url}`);
    // ensure temp directory exists
    if (!fs.existsSync(TEMP_DIR)) {
        fs.mkdirSync(TEMP_DIR, { recursive: true });
    }

    // extract filename from URL
    const filename = path.basename(new URL(url).pathname);
    const filepath = path.join(TEMP_DIR, filename);

    // download the file
    const response = await fetch(url);
    if (!response.ok) {
        throw new Error(`failed to download ${url}: ${response.status}`);
    }
    const buffer = Buffer.from(await response.arrayBuffer());
    fs.writeFileSync(filepath, buffer);
    const output = execSync(`sha256sum "${filepath}"`, { encoding: "utf-8" });
    let hash = output.split(/\s+/)[0];
    if (hash.startsWith('\\')) { hash = hash.substring(1); }
    fs.unlinkSync(filepath);
    return hash;
};

/** Parse a TOML string value, unescaping as needed */
export const from_toml_string = (toml_value: string): string => {
    // triple-quoted literal string '''...'''
    if (toml_value.startsWith("'''") && toml_value.endsWith("'''")) {
        return toml_value.slice(3, -3);
    }
    // single-quoted literal string '...' (no escaping)
    if (toml_value.startsWith("'") && toml_value.endsWith("'")) {
        return toml_value.slice(1, -1);
    }
    // double-quoted basic string "..." (with escape sequences)
    if (toml_value.startsWith('"') && toml_value.endsWith('"')) {
        const inner = toml_value.slice(1, -1);
        return inner
            .replace(/\\n/g, "\n")
            .replace(/\\r/g, "\r")
            .replace(/\\t/g, "\t")
            .replace(/\\\\/g, "\\")
            .replace(/\\"/g, '"');
    }
    // unquoted value (shouldn't happen for strings, but handle gracefully)
    return toml_value;
};

/** Format a string as a TOML value, using raw literal if escaping would be needed */
export const to_toml_string = (str: string): string => {
    const needs_escaping = str.includes("\\") || str.includes('"') || str.includes("\n") || str.includes("\r") || str.includes("\t");
    if (!needs_escaping) {
        return `"${str}"`;
    }
    if (!str.includes("'")) {
        return `'${str}'`;
    }
    if (!str.includes("'''")) {
        return `'''${str}'''`;
    }
    throw new Error("why does the input have triple single quote");
};
