//
// Copyright (C) 2025 Eric Le Bihan <eric.le.bihan.dev@free.fr>
//
// SPDX-License-Identifier: MIT
//

use cargo_metadata::{MetadataCommand, Package, PackageId};
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use xshell::Shell;

/// Source of a package.
#[derive(Debug)]
struct Source<'s>(&'s cargo_metadata::Source);

impl<'s> Source<'s> {
    /// Sanitize.
    fn sanitize(&self) -> Option<String> {
        if let Some((prefix, rest)) = self.0.to_string().split_once('+') {
            match prefix {
                "registry" => {
                    if rest.contains("crates.io-index") {
                        Some("crates-io".to_string())
                    } else {
                        Some(rest.to_string())
                    }
                }
                "git" => {
                    let bare = rest
                        .split('#')
                        .next()
                        .unwrap_or(rest)
                        .split('?')
                        .next()
                        .unwrap_or(rest);
                    Some(bare.to_string())
                }
                _ => None,
            }
        } else {
            None
        }
    }
}
/// Collect dependencies and override them with local sources.
#[derive(Debug)]
struct Overrider {
    manifest: PathBuf,
    sections: BTreeMap<String, BTreeSet<String>>,
}

impl Overrider {
    /// Create a new overrider for project with manifest at `path`.
    fn new<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        let manifest = path.as_ref().to_path_buf();
        let metadata =
            MetadataCommand::new().manifest_path(&manifest).exec()?;
        let pkg_by_id: BTreeMap<PackageId, &Package> = metadata
            .packages
            .iter()
            .map(|p| (p.id.clone(), p))
            .collect();
        let resolve = metadata.resolve.as_ref().ok_or("No resolve graph")?;
        let node_by_id: BTreeMap<_, _> =
            resolve.nodes.iter().map(|n| (n.id.clone(), n)).collect();
        let members: BTreeSet<_> =
            metadata.workspace_members.iter().cloned().collect();
        let mut direct_deps: BTreeSet<PackageId> = BTreeSet::new();
        for member in &members {
            if let Some(node) = node_by_id.get(member) {
                for dep in &node.deps {
                    direct_deps.insert(dep.pkg.clone());
                }
            }
        }
        let mut sections: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        for dep in &direct_deps {
            let pkg = pkg_by_id.get(dep).ok_or("Unknown package")?;
            if let Some(source) = &pkg.source {
                if let Some(source) = Source(source).sanitize() {
                    sections
                        .entry(source)
                        .or_default()
                        .insert(pkg.name.to_string());
                }
            }
        }
        Ok(Self { manifest, sections })
    }
    /// Override `package` with local sources at `path`.
    fn r#override<P: AsRef<Path>>(
        &self,
        package: &str,
        path: P,
    ) -> Result<(), Box<dyn Error>> {
        for (section, packages) in &self.sections {
            if packages.contains(package) {
                let header = if section == "crates-io" {
                    "[patch.crates.io]".to_string()
                } else {
                    format!("[patch.\"{section}\"]")
                };
                let text = format!(
                    "\n{header}\n{package} = {{ path = \"{}\" }}\n",
                    path.as_ref().display()
                );
                self.append_manifest(&text)?;
                return Ok(());
            }
        }
        Err(format!("Not found: {package}").into())
    }
    /// Append `text` to the manifest.
    fn append_manifest(&self, text: &str) -> std::io::Result<()> {
        let mut file = File::options().append(true).open(&self.manifest)?;
        file.write_all(text.as_bytes())
    }
}

/// Override `pkg` using local source at `path`
pub(super) fn r#override<P: AsRef<Path>>(
    _shell: &Shell,
    pkg: &str,
    path: P,
) -> Result<(), Box<dyn Error>> {
    println!("🔧 Overriding dependency '{pkg}'...");
    let manifest = std::env::current_dir()?.join("Cargo.toml");
    let overrider = Overrider::new(manifest)?;
    overrider.r#override(pkg, path)
}
