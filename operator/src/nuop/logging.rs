use std::{env, str::FromStr};

use tracing::Level;

use super::constants::{LOG_FORMAT, LOG_LEVEL};

pub fn init() {
    let log_level = Level::from_str(&env::var(LOG_LEVEL).unwrap_or_else(|_| "INFO".to_string()))
        .unwrap_or(Level::INFO);
    let log_format = env::var(LOG_FORMAT).unwrap_or_else(|_| "plain".to_string());
    let subscriber_builder = tracing_subscriber::fmt().with_max_level(log_level);
    if log_format.to_lowercase() == "json" {
        subscriber_builder.json().init();
    } else {
        subscriber_builder.init();
    }
}
