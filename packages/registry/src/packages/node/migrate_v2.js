function main(config) {
    config = config || {};
    if ("default-version" in config) {
        delete config["default-version"]["pnpm"];
    }
    return config;
}
