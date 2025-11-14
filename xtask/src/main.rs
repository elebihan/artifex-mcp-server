//
// Copyright (C) 2025 Eric Le Bihan <eric.le.bihan.dev@free.fr>
//
// SPDX-License-Identifier: MIT
//

mod prepare;

use prepare::prepare;
use std::{env, error::Error};
use xshell::Shell;

fn usage() {
    eprintln!(
        r#"Tasks:

prepare  Prepare development environment
"#
    );
}

fn main() -> Result<(), Box<dyn Error>> {
    let shell = Shell::new()?;
    let task = env::args().nth(1);
    match task.as_deref() {
        Some("prepare") => prepare(&shell)?,
        _ => usage(),
    }
    Ok(())
}
