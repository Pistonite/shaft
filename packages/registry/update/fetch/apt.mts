import type { MetaKeyValues } from "../metafile.mts";
import { fetch_with_retry } from "../util.mts";

const LAUNCHPAD_API = "https://api.launchpad.net/1.0/ubuntu/";
/** Ubuntu release to query: the latest LTS (26.04) */
const UBUNTU_SERIES = "resolute";
/** Pockets enabled by default on an Ubuntu install, so Proposed and Backports are excluded */
const DEFAULT_POCKETS = ["Release", "Security", "Updates"];

export type AptArgs = {
    package: string;
    query: (version: string) => Promise<MetaKeyValues> | MetaKeyValues;
};
type LaunchpadBinariesResponse = { entries: { binary_package_version: string; pocket: string }[] };
/** Fetch package version from the Ubuntu archive, as reported by apt/dpkg on amd64 */
export const fetch_from_apt = async ({ package: pkg, query }: AptArgs): Promise<MetaKeyValues> => {
    console.log(`-- fetching from apt: ${pkg}`);
    const params = new URLSearchParams({
        "ws.op": "getPublishedBinaries",
        binary_name: pkg,
        exact_match: "true",
        status: "Published",
        order_by_date: "false", // sort by version, newest first
        distro_arch_series: `${LAUNCHPAD_API}${UBUNTU_SERIES}/amd64`,
    });
    const response = await fetch_with_retry(`${LAUNCHPAD_API}+archive/primary?${params}`);
    if (!response.ok) {
        throw new Error(`failed to fetch apt package ${pkg}: ${response.status}`);
    }
    const data = await response.json() as LaunchpadBinariesResponse;
    const match = data.entries.find(e => DEFAULT_POCKETS.includes(e.pocket));
    if (!match) {
        throw new Error(`apt package not found: ${pkg}`);
    }
    const version = match.binary_package_version;
    console.log(`-- -- latest version of ${pkg} on apt: ${version}`);
    return await query(version);
};
