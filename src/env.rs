use std::env;

#[derive(Debug, Clone)]
pub struct Environment {
    pub os: String,
    pub hostname: String,
    pub arch: String,
}

impl Environment {
    pub fn current() -> Self {
        let hostname = hostname::get()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();

        Self {
            os: env::consts::OS.to_string(),
            hostname,
            arch: env::consts::ARCH.to_string(),
        }
    }
}
