use std::path::PathBuf;

use crate::{ItemId, TreeId};

#[derive(Debug, thiserror::Error)]
pub enum ContentError {
    #[error("cannot read {}", .path.display())]
    Read {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("{file} is not valid: {source}")]
    Parse {
        file: String,
        #[source]
        source: Box<ron::error::SpannedError>,
    },
    #[error("the item {item} is not valid: {problem}")]
    InvalidItem { item: ItemId, problem: &'static str },
    #[error("the tree {tree} is not valid: {problem}")]
    InvalidTree { tree: TreeId, problem: &'static str },
    #[error("the tree {tree} gives the unknown item {item}")]
    UnknownItem { tree: TreeId, item: ItemId },
    #[error("a landmark names the unknown tree {tree}")]
    UnknownLandmarkTree { tree: TreeId },
    #[error("a landmark of the tree {tree} has a position that is not finite")]
    LandmarkNotFinite { tree: TreeId },
}
