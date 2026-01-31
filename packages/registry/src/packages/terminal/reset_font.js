function main(config) {
    if (!is_object(config)) {
        config = {};
    }
    ensure_object(config, "profiles");
    ensure_object(config.profiles, "defaults");
    ensure_object(config.profiles.defaults, "font");
    config.profiles.defaults.font.face = "Consolas";
    return config;
}
