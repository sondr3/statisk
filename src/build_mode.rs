use clap::ValueEnum;
use serde::Serialize;
use std::fmt::Display;

#[derive(Debug, Copy, Clone, ValueEnum, Serialize)]
pub enum BuildMode {
    Optimized,
    Normal,
}

impl Display for BuildMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BuildMode::Optimized => write!(f, "optimized"),
            BuildMode::Normal => write!(f, "normal"),
        }
    }
}

impl BuildMode {
    #[must_use]
    pub const fn optimize(&self) -> bool {
        matches!(self, Self::Optimized)
    }

    #[must_use]
    pub const fn normal(&self) -> bool {
        matches!(self, Self::Normal)
    }
}
