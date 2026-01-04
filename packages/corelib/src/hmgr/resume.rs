use crate::hmgr;

/// Save a the current command to HOME/.interrupted
pub fn mark_interrupted() {
    let path = hmgr::paths::dot_interrupted();
    let command = shell_words::join(std::env::args());
    if let Err(e) = cu::fs::write(path, &command) {
        cu::error!("{e:?}");
        cu::warn!(
            "failed to store interrupted file, cannot automatically resume the same command."
        );
    };
}

/// If a command was previously interrupted, read the raw command args and structured
/// command data from HOME/previous_command.json
pub fn extract_previously_interrupted_json_command() -> Option<(String, String)> {
    let path = hmgr::paths::dot_interrupted();
    if !path.exists() {
        return None;
    }
    let previous_command_args =
        cu::fs::read_string(&path).unwrap_or("(unknown command)".to_string());
    let command_file = hmgr::paths::previous_command_json();
    if !command_file.exists() {
        return None;
    }
    let json = cu::fs::read_string(command_file).ok();
    if json.is_some() {
        if let Err(e) = cu::fs::remove(path) {
            cu::error!("{e:?}");
            cu::warn!("failed to clean up interrupt marker");
        }
    }
    json.map(|json| (previous_command_args, json))
}

/// Save the serialized command data to HOME/previous_command.json
pub fn save_command_json(content: &str) {
    let command_file = hmgr::paths::previous_command_json();
    if let Err(e) = cu::fs::write(command_file, content) {
        cu::error!("{e:?}");
        cu::warn!("failed to store command, cannot automatically resume if interrupted by command");
    }
}
