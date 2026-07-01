import type { MetaKeyValues } from "../metafile.mts";
import { fetch_with_retry } from "../util.mts";

const HOMEBREW_API = "https://formulae.brew.sh/api/";

export type HomebrewArgs = {
    package: string;
    query: (version: string) => Promise<MetaKeyValues> | MetaKeyValues;
};
type HomebrewFormulaResponse = { versions: { stable: string } };
/** Fetch package version from Homebrew formulae */
export const fetch_from_homebrew = async ({ package: pkg, query }: HomebrewArgs): Promise<MetaKeyValues> => {
    console.log(`-- fetching from homebrew: ${pkg}`);
    const response = await fetch_with_retry(`${HOMEBREW_API}formula/${encodeURIComponent(pkg)}.json`);
    if (!response.ok) {
        throw new Error(`failed to fetch homebrew package ${pkg}: ${response.status}`);
    }
    const data = await response.json() as HomebrewFormulaResponse;
    const version = data.versions.stable;
    console.log(`-- -- latest version of ${pkg} on homebrew: ${version}`);
    return await query(version);
};

export type HomebrewCaskArgs = {
    cask: string;
    query: (version: string) => Promise<MetaKeyValues> | MetaKeyValues;
};
type HomebrewCaskResponse = { version: string };
/** Fetch package version from Homebrew casks */
export const fetch_from_homebrew_cask = async ({ cask, query }: HomebrewCaskArgs): Promise<MetaKeyValues> => {
    console.log(`-- fetching from homebrew cask: ${cask}`);
    const response = await fetch_with_retry(`${HOMEBREW_API}cask/${encodeURIComponent(cask)}.json`);
    if (!response.ok) {
        throw new Error(`failed to fetch homebrew cask ${cask}: ${response.status}`);
    }
    const data = await response.json() as HomebrewCaskResponse;
    const version = data.version;
    console.log(`-- -- latest version of ${cask} on homebrew cask: ${version}`);
    return await query(version);
};
