use std::path::Path;

use bevy::math::Vec3;
use bevy::prelude::Resource;
use serde::Deserialize;

use crate::content::{parse, read};
use crate::{Content, ContentError, TreeId};

const LANDMARKS_FILE: &str = "landmarks.ron";

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TreePlacement {
    pub tree: TreeId,
    pub position: Vec3,
}

#[derive(Resource, Debug, Clone, PartialEq, Default)]
pub struct Landmarks {
    trees: Vec<TreePlacement>,
}

impl Landmarks {
    pub fn load(directory: &Path, content: &Content) -> Result<Self, ContentError> {
        Self::from_source(&read(&directory.join(LANDMARKS_FILE))?, content)
    }

    pub fn from_source(source: &str, content: &Content) -> Result<Self, ContentError> {
        let trees: Vec<TreePlacement> = parse(LANDMARKS_FILE, source)?;
        for placement in &trees {
            if content.tree(&placement.tree).is_none() {
                return Err(ContentError::UnknownLandmarkTree {
                    tree: placement.tree.clone(),
                });
            }
            if !placement.position.is_finite() {
                return Err(ContentError::LandmarkNotFinite {
                    tree: placement.tree.clone(),
                });
            }
        }
        Ok(Self { trees })
    }

    #[must_use]
    pub fn trees(&self) -> &[TreePlacement] {
        &self.trees
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use bevy::math::Vec3;

    use super::Landmarks;
    use crate::{Content, ContentError, TreeId};

    fn content() -> Content {
        Content::from_sources(
            "{ \"oak_logs\": (stack_size: 1) }",
            "{ \"oak\": (level: 1, log: \"oak_logs\", experience: 25, seconds_per_log: 3.0, logs: 3, regrow_seconds: 30.0) }",
        )
        .unwrap()
    }

    #[test]
    fn the_repository_landmarks_are_valid() {
        let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../content");
        let content = Content::load(&directory).unwrap();

        let landmarks = Landmarks::load(&directory, &content).unwrap();

        assert_ne!(landmarks.trees().len(), 0);
    }

    #[test]
    fn a_landmark_gives_its_tree_and_its_position() {
        let landmarks =
            Landmarks::from_source("[(tree: \"oak\", position: (4.0, 0.0, -3.0))]", &content())
                .unwrap();

        assert_eq!(landmarks.trees()[0].tree, TreeId::new("oak"));
        assert_eq!(landmarks.trees()[0].position, Vec3::new(4.0, 0.0, -3.0));
    }

    #[test]
    fn a_landmark_with_an_unknown_tree_is_an_error() {
        let error =
            Landmarks::from_source("[(tree: \"ash\", position: (0.0, 0.0, 0.0))]", &content())
                .unwrap_err();

        assert_eq!(error.to_string(), "a landmark names the unknown tree ash");
    }

    #[test]
    fn a_landmark_position_that_is_not_finite_is_an_error() {
        let error =
            Landmarks::from_source("[(tree: \"oak\", position: (NaN, 0.0, 0.0))]", &content())
                .unwrap_err();

        assert_eq!(
            error.to_string(),
            "a landmark of the tree oak has a position that is not finite"
        );
    }

    #[test]
    fn an_unknown_field_is_an_error() {
        let source = "[(tree: \"oak\", position: (0.0, 0.0, 0.0), scale: 2.0)]";

        assert!(matches!(
            Landmarks::from_source(source, &content()),
            Err(ContentError::Parse { .. })
        ));
    }
}
