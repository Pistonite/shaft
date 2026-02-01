import { GITHUB_API, parse_github_repo } from "../util.mts";

type GitHubRepoResponse = { default_branch: string };
type GitHubBranchResponse = { commit: { sha: string } };

/** Fetch the latest commit hash of the default branch */
export const fetch_latest_commit = async (repo: string): Promise<string> => {
    const repo_path = parse_github_repo(repo);
    console.log(`-- fetching latest commit for ${repo_path}`);
    // get repo info to find default branch
    const repo_response = await fetch(`${GITHUB_API}repos/${repo_path}`);
    if (!repo_response.ok) {
        throw new Error(`failed to fetch repo info for ${repo_path}: ${repo_response.status}`);
    }
    const repo_data = await repo_response.json() as GitHubRepoResponse;
    const default_branch = repo_data.default_branch;
    // get the latest commit on the default branch
    const branch_response = await fetch(`${GITHUB_API}repos/${repo_path}/branches/${default_branch}`);
    if (!branch_response.ok) {
        throw new Error(`failed to fetch branch ${default_branch} for ${repo_path}: ${branch_response.status}`);
    }
    const branch_data = await branch_response.json() as GitHubBranchResponse;
    const commit = branch_data.commit.sha;
    console.log(`-- -- latest commit of ${repo_path} on ${default_branch}: ${commit}`);
    return commit;
};
