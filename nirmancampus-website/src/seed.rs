//! Seed public website media into the filesystem plugin.
//!
//! Template assets are embedded. Any leftover tree under `nirman_campus/` or
//! `[p_nirmancampus_website] staticDir` is imported once into `website/static/`.

use std::path::{Path, PathBuf};

use lariv_rs::plugins::filesystem::{
    entities::VNode,
    node::{self, NodeFile},
    state::FilesystemState,
};
use tokio::io::AsyncReadExt;

use crate::state::WebsiteState;

const VNODE_ROOT: &[&str] = &["website", "static"];

struct EmbeddedAsset {
    relative: &'static str,
    bytes: &'static [u8],
}

const EMBEDDED_ASSETS: &[EmbeddedAsset] = &[
    EmbeddedAsset {
        relative: "images/logo.png",
        bytes: include_bytes!("../assets/static/images/logo.png"),
    },
    EmbeddedAsset {
        relative: "images/hero.jpg",
        bytes: include_bytes!("../assets/static/images/hero.jpg"),
    },
    EmbeddedAsset {
        relative: "images/kansalfoundationwpic.jpeg",
        bytes: include_bytes!("../assets/static/images/kansalfoundationwpic.jpeg"),
    },
];

pub async fn ensure_website_static(
    website: &WebsiteState,
    fs: &FilesystemState,
) -> anyhow::Result<()> {
    for asset in EMBEDDED_ASSETS {
        ensure_file_bytes(fs, asset.relative, asset.bytes, true).await?;
    }

    let mut imported = std::collections::BTreeSet::new();
    for dir in import_directories(&website.static_dir) {
        let Ok(canon) = dir.canonicalize() else {
            continue;
        };
        if !imported.insert(canon.clone()) {
            continue;
        }
        import_tree(fs, &canon).await?;
    }
    Ok(())
}

fn import_directories(configured: &str) -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    let configured = configured.trim();
    if !configured.is_empty() {
        dirs.push(resolve_dir(configured));
    }
    dirs.push(PathBuf::from("nirman_campus"));
    if let Some(exe_dir) = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join("nirman_campus")))
    {
        dirs.push(exe_dir);
    }
    dirs.into_iter().filter(|d| d.is_dir()).collect()
}

fn resolve_dir(configured: &str) -> PathBuf {
    let path = PathBuf::from(configured);
    if path.is_absolute() {
        return path;
    }
    if let Some(exe_dir) = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(Path::to_path_buf))
    {
        let next_to_exe = exe_dir.join(&path);
        if next_to_exe.is_dir() {
            return next_to_exe;
        }
    }
    path
}

async fn import_tree(fs: &FilesystemState, root: &Path) -> anyhow::Result<()> {
    let mut files = Vec::new();
    collect_files(root, root, &mut files)?;
    tracing::info!(
        root = %root.display(),
        count = files.len(),
        "website: importing files into filesystem plugin"
    );
    for (relative, path) in files {
        let bytes = tokio::fs::read(&path)
            .await
            .map_err(|e| anyhow::anyhow!("read {}: {e}", path.display()))?;
        ensure_file_bytes(fs, &relative, &bytes, false).await?;
    }
    Ok(())
}

fn collect_files(root: &Path, dir: &Path, out: &mut Vec<(String, PathBuf)>) -> anyhow::Result<()> {
    let entries =
        std::fs::read_dir(dir).map_err(|e| anyhow::anyhow!("read dir {}: {e}", dir.display()))?;
    for entry in entries {
        let entry = entry.map_err(|e| anyhow::anyhow!("read dir entry: {e}"))?;
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name.starts_with('.') {
            continue;
        }
        let path = entry.path();
        let meta = entry
            .metadata()
            .map_err(|e| anyhow::anyhow!("stat {}: {e}", path.display()))?;
        if meta.is_dir() {
            collect_files(root, &path, out)?;
            continue;
        }
        if !meta.is_file() {
            continue;
        }
        let rel = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");
        if rel.is_empty() {
            continue;
        }
        out.push((rel, path));
    }
    Ok(())
}

async fn ensure_file_bytes(
    fs: &FilesystemState,
    relative: &str,
    bytes: &[u8],
    overwrite: bool,
) -> anyhow::Result<()> {
    let requested = split_relative(relative)?;
    let Some((name, rest)) = requested.split_last() else {
        return Ok(());
    };
    let mut dir_segments: Vec<String> = VNODE_ROOT.iter().map(|s| (*s).to_string()).collect();
    dir_segments.extend(rest.iter().cloned());
    let parent_id = node::ensure_directory_path(&fs.db, fs.store.as_ref(), None, &dir_segments)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    let parent = match parent_id {
        Some(id) => node::get_by_id(&fs.db, id)
            .await
            .map_err(|e| anyhow::anyhow!("{e}"))?,
        None => None,
    };
    if let Some(existing) = node::find_child(&fs.db, parent_id, name, false).await? {
        if !overwrite {
            return Ok(());
        }
        if vnode_bytes_match(fs, &existing, bytes).await? {
            return Ok(());
        }
        node::update(
            &fs.db,
            fs.store.as_ref(),
            existing,
            name.clone(),
            Some(NodeFile::Bytes {
                filename: name.clone(),
                data: bytes.to_vec(),
            }),
        )
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;
        tracing::info!(relative, "website: updated filesystem vnode");
        return Ok(());
    }

    node::create(
        &fs.db,
        fs.store.as_ref(),
        name.clone(),
        false,
        Some(NodeFile::Bytes {
            filename: name.clone(),
            data: bytes.to_vec(),
        }),
        parent.as_ref(),
    )
    .await
    .map_err(|e| anyhow::anyhow!("{e}"))?;
    tracing::info!(relative, "website: created filesystem vnode");
    Ok(())
}

async fn vnode_bytes_match(
    fs: &FilesystemState,
    existing: &VNode,
    bytes: &[u8],
) -> anyhow::Result<bool> {
    let path = existing.file_path.as_deref().unwrap_or("");
    let mut download = match fs.store.open(path, &existing.name).await {
        Ok(d) => d,
        Err(e) if e.is_missing() => return Ok(false),
        Err(e) => return Err(anyhow::anyhow!("{e}")),
    };
    let mut current = Vec::new();
    download.reader.read_to_end(&mut current).await?;
    Ok(current == bytes)
}

pub fn split_relative(relative: &str) -> anyhow::Result<Vec<String>> {
    let mut parts = Vec::new();
    for part in relative.split(['/', '\\']) {
        if part.is_empty() || part == "." {
            continue;
        }
        if part == ".." || part.starts_with('.') {
            anyhow::bail!("invalid static path {relative}");
        }
        parts.push(part.to_string());
    }
    if parts.is_empty() {
        anyhow::bail!("empty static path");
    }
    Ok(parts)
}

pub async fn find_static_vnode(
    db: &sea_orm::DatabaseConnection,
    relative: &str,
) -> anyhow::Result<Option<VNode>> {
    let requested = split_relative(relative)?;
    let mut segments: Vec<String> = VNODE_ROOT.iter().map(|s| (*s).to_string()).collect();
    segments.extend(requested);
    let Some((name, dirs)) = segments.split_last() else {
        return Ok(None);
    };
    let mut parent_id = None;
    for dir in dirs {
        let Some(node) = node::find_child(db, parent_id, dir, true).await? else {
            return Ok(None);
        };
        parent_id = Some(node.id);
    }
    Ok(node::find_child(db, parent_id, name, false).await?)
}

#[cfg(test)]
mod tests {
    use super::split_relative;

    #[test]
    fn split_relative_rejects_parent() {
        assert!(split_relative("../secret").is_err());
        assert!(split_relative("images/logo.png").is_ok());
    }
}
