//
// This file is part of artifex-mcp-server
//
// Copyright (C) 2025 Eric Le Bihan <eric.le.bihan.dev@free.fr>
//
// SPDX-License-Identifier: MIT
//

use artifex_engine::Config as EngineConfig;
use serde::Deserialize;
use std::path::Path;
use thiserror::Error;

/// Errors reported when handling configuration.
#[derive(Debug, Error)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("TOML error: {0}")]
    Toml(#[from] toml::de::Error),
}

/// Configuration of MCP server.
#[derive(Clone, Debug, Default, Deserialize)]
pub struct Config {
    /// Artifex engine configuration.
    pub engine: EngineConfig,
}

impl Config {
    /// Create a configuration from file at `path`.
    pub fn with_path<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let text = std::fs::read_to_string(path)?;
        let config = toml::de::from_str(&text)?;
        Ok(config)
    }
}
