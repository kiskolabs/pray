use crate::client_trust::env_truthy;

pub fn color_enabled() -> bool {
    if env_truthy("PRAY_NO_COLOR") || no_color_requested() {
        return false;
    }
    std::env::var("TERM")
        .ok()
        .is_none_or(|value| value != "dumb")
}

pub fn no_color_requested() -> bool {
    std::env::var("NO_COLOR")
        .map(|value| !value.is_empty())
        .unwrap_or(false)
}

pub fn no_input_requested() -> bool {
    env_truthy("PRAY_NO_INPUT")
}
