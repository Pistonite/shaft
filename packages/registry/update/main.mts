/**
 * Update metadata.toml
 *
 * Usage: node --experimental-strip-types update-metadata.ts [PACKAGE]
 *
 * PACKAGE is the package to update, or all packages if omitted
 */

//* To get type checking in your IDE, install @types/node with a package manager */
/// <reference types="node" />
import { Metafile, MetafilePkg, type MetaKeyValues } from "./metafile.mts";
import { METADATA_TOML } from "./util.mts";

import * as Packages from "./packages.mts";
import type { PackageFn } from "./packages.mts";

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

type UpdatePayload = [string, MetaKeyValues];

const fetch_by_package = async (meta: Metafile, pkg: string): Promise<MetaKeyValues> => {
    console.log(`fetching '${pkg}'...`);
    const meta_pkg = new MetafilePkg(meta, pkg);
    const fetch_fn = Packages[`pkg_${pkg}`] as PackageFn;
    if (!fetch_fn) {
        console.log(`WARNING: unknown package '${pkg}'`);
        return {};
    }
    const result = await fetch_fn(meta_pkg);
    if (!Array.isArray(result)) {
        return result;
    }
    const results = await Promise.allSettled(result);
    let has_error = false;
    let output: MetaKeyValues = {};
    for (const result of results) {
        if (result.status === "rejected") {
            has_error = true;
            const message = result.reason instanceof Error ? result.reason.message : result;
            console.log(`-- ERROR: fetching pkg ${pkg}: ${message}`);
        } else {
            output = { ...output, ...result.value };
        }
    }
    return output;
}

void main();
