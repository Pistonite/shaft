/// <reference types="node" />
import * as fs from "node:fs";

import { from_toml_string, to_toml_string } from "./util.mts";

export class MetafilePkg {
    public file: Metafile;
    public pkg: string;
    constructor(file: Metafile, pkg: string) {
        this.file = file;
        this.pkg = pkg;
    }
    public get(key: string): string { return this.file.get(this.pkg, key); }
    public repo(): string { return this.get("REPO"); }
}

export class Metafile {
    private path: string;
    private sections: MetaSection[];

    constructor(path: string){ 
        this.path = path
        this.sections = [];
    }

    public open() {
        const content = fs.readFileSync(this.path, "utf-8");
        const lines = content.split("\n").map(x => x.trimEnd());
        const sections: MetaSection[] = [];
        let curr_section: MetaSection = { name:"", storage:[] };
        for (const line of lines) {
            const trimmed = line.trim();
            if (trimmed.startsWith("[") && trimmed.endsWith("]")) {
                if (curr_section.name) {
                    sections.push(curr_section);
                    curr_section = { name:"", storage:[] };
                }
                curr_section.name = trimmed.slice(1, -1);
                continue;
            }
            curr_section.storage.push(line.trimEnd());
        }
        if (curr_section.name) {
            sections.push(curr_section);
        }
        this.sections = sections;
    }

    public pkg_names(): string[] { return this.sections.map(x => x.name); }

    public save() {
        const lines: string[] = [];
        for (const section of this.sections) {
            lines.push(`[${section.name}]`);
            lines.push(...section.storage);
        }
        fs.writeFileSync(this.path, lines.join("\n"), "utf-8");
    }

    public get(pkg: string, key: string): string {
        const section = this.section_by_name(pkg);
        const [i, value] = storage_find_by_key(section.storage, key);
        if (i === -1) { throw new Error(`cannot find key ${pkg}.${key}`); }
        return from_toml_string(value);
    }

    public update(pkg: string, update: MetaKeyValues): boolean {
        let updated = false;
        const section = this.section_by_name(pkg);
        for (const key in update) {
            const value = update[key];
            updated = storage_update_by_key(section, key, value)||updated;
        }
        return updated;
    }

    private section_by_name(name: string): MetaSection {
        const s = this.sections.find(x=>x.name===name);
        if (!s) { throw new Error(`cannot find section [${name}]`); }
        return s;
    }
}
type MetaSection = {
    name: string,
    storage: MetaStorage
}
type MetaStorage = string[];
export type MetaKeyValues = Record<string, string>;

/** Update a key's value. return true if was updated, false if old value was the same */
const storage_update_by_key = (section: MetaSection, key: string, value: string): boolean => {
    const [i, old_value_raw] = storage_find_by_key(section.storage, key);
    if (i === -1) { throw new Error(`cannot find key ${section.name}.${key}`); }
    const old_value = from_toml_string(old_value_raw);
    if (old_value === value) { return false };
    const new_value_raw = to_toml_string(value);
    console.log(`update: [${section.name}] ${key} = ${old_value_raw} -> ${new_value_raw}`);
    section.storage[i] = `${key} = ${new_value_raw}`;
    return true;
}

/** Find a key's line index within section bounds, returns -1 if not found. Second return is the value */
const storage_find_by_key = (
    storage: MetaStorage,
    key: string
): [number, string] => {
    for (let i = 0; i < storage.length; i++) {
        const trimmed = storage[i].trim();
        if (trimmed === "" || trimmed.startsWith("#")) {
            continue;
        }
        if (trimmed.startsWith(key)) {
            const rest = trimmed.slice(key.length).trimStart();
            if (rest.startsWith("=")) {
                return [i, rest.substring(1).trim()];
            }
        }
    }
    return [-1, ""];
};
