import type { MetaKeyValues } from "../metafile.mts";
import { CRATESIO_API } from "../util.mts";

export type CratesIoArgs = {
    crate: string;
    query?: (version: string) => Promise<MetaKeyValues> | MetaKeyValues;
};
type CratesIoResponse = { crate: { newest_version: string } };
/** Fetch package updates from crates.io */
export const fetch_from_cratesio = async ({ crate, query }: CratesIoArgs): Promise<MetaKeyValues> => {
    query = query || ((VERSION) => ({ VERSION }));
    console.log(`-- fetching from crates.io: ${crate}`);
    const response = await fetch(`${CRATESIO_API}crates/${crate}`, {
        headers: { "User-Agent": "shaft-registry-updater" } // crates.io requires UA
    });
    if (!response.ok) {
        throw new Error(`failed to fetch crate ${crate}: ${response.status}`);
    }
    const data = await response.json() as CratesIoResponse;
    const version = data.crate.newest_version;
    console.log(`-- -- latest version of ${crate}: ${version}`);
    return await query(version);
};
