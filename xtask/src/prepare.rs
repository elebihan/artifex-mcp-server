//
// Copyright (C) 2025 Eric Le Bihan <eric.le.bihan.dev@free.fr>
//
// SPDX-License-Identifier: MIT
//

#[cfg(unix)]
use std::os::unix::prelude::PermissionsExt;
use std::{error::Error, fs};

use xshell::Shell;

fn install_hooks(shell: &Shell) -> Result<(), Box<dyn Error>> {
    let cwd = shell.current_dir();
    let src = cwd.join("hooks");
    let hooks = fs::read_dir(src)?
        .filter_map(|e| e.ok())
        .filter_map(|e| match e.file_type() {
            Ok(t) if t.is_file() => Some(e),
            _ => None,
        })
        .filter_map(|e| {
            let path = e.path();
            match path.extension() {
                Some(e) if e == "hook" => Some(path),
                _ => None,
            }
        });
    for hook in hooks {
        let name = hook
            .file_stem()
            .ok_or_else(|| "Missing file stem".to_string())?;
        let mut dst = cwd.join(".git");
        dst.push("hooks");
        dst.push(name);
        println!("📂 Installing Git hook: {}", &hook.display());
        shell.copy_file(hook, &dst)?;
        #[cfg(unix)]
        {
            let mut permissions = fs::metadata(&dst)?.permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(dst, permissions)?;
        }
    }
    Ok(())
}

pub(super) fn prepare(shell: &Shell) -> Result<(), Box<dyn Error>> {
    println!("🥁 Preparing build environment...");
    install_hooks(shell)
}
