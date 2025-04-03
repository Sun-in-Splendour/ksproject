use std::path::PathBuf;

#[derive(Default)]
pub enum Input {
    #[default]
    Stdin,
    String(String),
    File(PathBuf),
}

#[derive(Default)]
pub enum Output {
    #[default]
    Stdout,
    Stderr,
    File(PathBuf),
}

#[derive(Default)]
pub enum Format {
    #[default]
    Json,
    Text,
    Html,
    Debug,
}

#[derive(Default)]
pub enum Level {
    #[default]
    Fatal,
    Debug,
    Warning,
    Error,
}

impl Level {
    pub const fn error(&self) -> bool {
        matches!(self, Level::Debug | Level::Warning | Level::Error)
    }

    pub const fn warning(&self) -> bool {
        matches!(self, Level::Debug | Level::Warning)
    }
}

#[derive(Default)]
pub enum ErrorOutput {
    #[default]
    Stderr,
    Stdout,
    File(PathBuf),
}
