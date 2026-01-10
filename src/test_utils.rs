#![cfg(test)]

use std::path::Path;

pub enum FsNode<'a> {
    File(&'a str, &'a str),
    Dir(&'a str, Vec<FsNode<'a>>),
}

pub fn create_fs_tree(root: &Path, node: &FsNode) -> std::io::Result<()> {
    match node {
        FsNode::File(name, contents) => {
            std::fs::write(root.join(name), contents)?;
        }
        FsNode::Dir(name, children) => {
            let dir = root.join(name);
            std::fs::create_dir(&dir)?;
            for child in children {
                create_fs_tree(&dir, child)?;
            }
        }
    }
    Ok(())
}