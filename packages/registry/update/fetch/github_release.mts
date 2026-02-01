import type { MetaKeyValues } from "../metafile.mts";
import { GITHUB_API, parse_github_repo, sha256_from_url } from "../util.mts";

export type GitHubReleaseArgs = {
    /** repo to fetch from */
    repo: string;
    /** select a tag, omit to use latest */
    tag?: (tags: string[]) => string;
    /** query for meta key values */
    query: (repo: string, tag: string, artifacts: GitHubArtifactData[]) => Promise<MetaKeyValues> | MetaKeyValues;
} & Partial<(GitHubReleaseArgsArtifact | GitHubReleaseArgsArtifacts)>;
type GitHubReleaseArgsArtifacts = {
    /** select artifacts from the tag */
    artifacts: (tag: string) => string[];
    artifact: undefined;
};
type GitHubReleaseArgsArtifact = {
    /** select an artifact from all artifact names */
    artifact: (artifacts: string[]) => string[];
    artifacts: undefined;
};
type GitHubTagsResponse = { name: string }[];
type GitHubReleaseResponse = { tag_name: string; assets: { name: string }[] };
type GitHubArtifactData = {
    name: string;
    url: string;
    sha: string;
};

/** Fetch package updates from GitHub releases */
export const fetch_from_github_release = async ({ 
    repo, 
    tag: tag_picker, 
    artifacts: artifact_calc,
    artifact: artifact_picker,
    query,
}: GitHubReleaseArgs): Promise<MetaKeyValues> => {
    console.log(`-- fetching from github repo: ${repo}`);
    const repo_path = parse_github_repo(repo);

    let selected_tag: string;

    if (tag_picker) {
        // fetch all tags and let the picker choose
        const tags_response = await fetch(`${GITHUB_API}repos/${repo_path}/tags`);
        if (!tags_response.ok) {
            throw new Error(`failed to fetch tags for ${repo_path}: ${tags_response.status}`);
        }
        const tags_data = await tags_response.json() as GitHubTagsResponse;
        const tags = tags_data.map(t => t.name);
        if (tags.length === 0) {
            throw new Error(`no tags found for ${repo_path}`);
        }
        selected_tag = tag_picker(tags);
    } else {
        // fetch latest release
        const release_response = await fetch(`${GITHUB_API}repos/${repo_path}/releases/latest`);
        if (!release_response.ok) {
            throw new Error(`failed to fetch latest release for ${repo_path}: ${release_response.status}`);
        }
        const release_data = await release_response.json() as GitHubReleaseResponse;
        selected_tag = release_data.tag_name;
    }
    // select artifacts if any
    const artifact_names: string[] = [];
    let all = !!artifact_picker;
    if (artifact_calc) {
        artifact_names.push(...artifact_calc(selected_tag));
    }
    const artifacts: GitHubArtifactData[] = [];
    if (all || artifact_names.length) {
        // fetch the release for this tag to get artifacts
        console.log(`-- fetching release '${selected_tag}' from ${repo}`);
        const release_response = await fetch(`${GITHUB_API}repos/${repo_path}/releases/tags/${selected_tag}`);
        if (!release_response.ok) {
            throw new Error(`failed to fetch release ${selected_tag} for ${repo_path}: ${release_response.status}`);
        }
        const release_data = await release_response.json() as GitHubReleaseResponse;
        const actual_artifact_names = release_data.assets.map(a => a.name);
        if (all) {
            if (!artifact_picker) {
                throw new Error(`artifact picker must be provided when artifacts: "all"`);
            }
            artifact_names.push(...artifact_picker(actual_artifact_names));
        }
        const fetcher = async (artifact: string) => {
            if (!actual_artifact_names.includes(artifact)) {
                throw new Error(`artifact not found from '${repo}', release ${selected_tag}: ${artifact}`);
            }
            const url = `${repo}/releases/download/${selected_tag}/${artifact}`;
            const sha = await sha256_from_url(url);
            return { name: artifact, url, sha };
        };
        const results = await Promise.allSettled(artifact_names.map(fetcher));
        let has_error = false;
        for (const result of results) {
            if (result.status === "rejected") {
                has_error = true;
                const message = result.reason instanceof Error ? result.reason.message : result;
                console.log(`-- ERROR: fetching artifact: ${message}`);
            } else {
                const data = result.value;
                console.log(`-- -- fetched ${data.url}`);
                artifacts.push(data);
            }
        }
        if (has_error) {
            throw new Error("there were errors fetching release artifacts from "+repo);
        }
    }

    return await query(repo, selected_tag, artifacts);
};
