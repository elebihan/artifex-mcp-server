//
// Copyright (C) 2025 Eric Le Bihan <eric.le.bihan.dev@free.fr>
//
// SPDX-License-Identifier: MIT
//

mod r#override;
mod prepare;

use r#override::r#override;
use prepare::prepare;
use std::{env, error::Error};
use xshell::Shell;

fn usage() {
    eprintln!(
        r#"Tasks:

override Override dependency with local source
prepare  Prepare development environment
"#
    );
}

fn main() -> Result<(), Box<dyn Error>> {
    let shell = Shell::new()?;
    let mut args = env::args().skip(1);
    let task = args.next();
    match task.as_deref() {
        Some("override") => {
            let pkg = args.next().ok_or("Missing crate name")?;
            let path = args.next().ok_or("Missing local sources path")?;
            r#override(&shell, &pkg, &path)?;
        }
        Some("prepare") => prepare(&shell)?,
        _ => usage(),
    }
    Ok(())
}
