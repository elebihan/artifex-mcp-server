//
// This file is part of artifex-mcp-server
//
// Copyright (C) 2025 Eric Le Bihan <eric.le.bihan.dev@free.fr>
//
// SPDX-License-Identifier: MIT
//

use clap::{Parser, ValueEnum};
use std::path::PathBuf;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum TransportMode {
    /// Use HTTP.
    Http,
    /// Use standard I/O.
    Stdio,
}

#[derive(Debug, Parser)]
#[command(version, about = "Manage Google Keep notes")]
pub struct Cli {
    /// Path to configuration file.
    #[arg(short, long)]
    pub config: Option<PathBuf>,
    /// Transport to use.
    #[arg(short, long, value_enum, default_value_t = TransportMode::Http)]
    pub transport: TransportMode,
}

pub fn build_cli() -> Cli {
    Cli::parse()
}
