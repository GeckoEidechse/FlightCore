/// Prompt on plugin
/// Returns:
/// - true: user accepted plugin install
/// - false: user denied plugin install
pub fn plugin_prompt() -> bool {
    log::error!("Plugin install cancelled");
    // TODO handle plugin installation again
    false
}
