function main(config) {
    config = config || {};
    return {
        "default-version": {
            "node": config["node-version"] || "",
            "pnpm": config["pnpm-version"] || "",
            "yarn": config["yarn-version"] || "",
        }
    }
}
