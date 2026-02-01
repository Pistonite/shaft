import { GITHUB_API, parse_github_repo } from "../util.mts";

type GitHubReleaseResponse = { body: string, name: string };

/** Fetch the description of the release */
export const fetch_release_name = async (repo: string, tag: string): Promise<string> => {
    const repo_path = parse_github_repo(repo);
    console.log(`-- fetching release description for ${repo_path} tag ${tag}`);
    const release_response = await fetch(`${GITHUB_API}repos/${repo_path}/releases/tags/${tag}`);
    if (!release_response.ok) {
        throw new Error(`failed to fetch release ${tag} for ${repo_path}: ${release_response.status}`);
    }
    const release_data = await release_response.json() as GitHubReleaseResponse;
    return release_data.name;
};
