use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use bevy::prelude::Resource;
use serde::Deserialize;
use serde::de::DeserializeOwned;
use vespera_core::Level;

use crate::{ContentError, ItemId, TreeId};

const ITEMS_FILE: &str = "items.ron";
const TREES_FILE: &str = "trees.ron";

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ItemDefinition {
    pub stack_size: u32,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TreeDefinition {
    pub level: Level,
    pub log: ItemId,
    pub experience: u32,
    pub seconds_per_log: f32,
    pub logs: u8,
    pub regrow_seconds: f32,
}

#[derive(Resource, Debug, Clone, PartialEq)]
pub struct Content {
    items: BTreeMap<ItemId, ItemDefinition>,
    trees: BTreeMap<TreeId, TreeDefinition>,
}

impl Content {
    pub fn load(directory: &Path) -> Result<Self, ContentError> {
        let items = read(&directory.join(ITEMS_FILE))?;
        let trees = read(&directory.join(TREES_FILE))?;
        Self::from_sources(&items, &trees)
    }

    pub fn from_sources(items: &str, trees: &str) -> Result<Self, ContentError> {
        let content = Self {
            items: parse(ITEMS_FILE, items)?,
            trees: parse(TREES_FILE, trees)?,
        };
        content.validate()?;
        Ok(content)
    }

    #[must_use]
    pub fn item(&self, id: &ItemId) -> Option<&ItemDefinition> {
        self.items.get(id)
    }

    #[must_use]
    pub fn tree(&self, id: &TreeId) -> Option<&TreeDefinition> {
        self.trees.get(id)
    }

    fn validate(&self) -> Result<(), ContentError> {
        if let Some((item, _)) = self
            .items
            .iter()
            .find(|(_, definition)| definition.stack_size == 0)
        {
            return Err(ContentError::InvalidItem {
                item: item.clone(),
                problem: "the stack size must be at least 1",
            });
        }
        for (tree, definition) in &self.trees {
            if let Some(problem) = definition.problem() {
                return Err(ContentError::InvalidTree {
                    tree: tree.clone(),
                    problem,
                });
            }
            if !self.items.contains_key(&definition.log) {
                return Err(ContentError::UnknownItem {
                    tree: tree.clone(),
                    item: definition.log.clone(),
                });
            }
        }
        Ok(())
    }
}

impl TreeDefinition {
    fn problem(&self) -> Option<&'static str> {
        if self.logs == 0 {
            Some("it must give at least one log")
        } else if self.seconds_per_log <= 0.0 || self.regrow_seconds <= 0.0 {
            Some("its times must be more than zero")
        } else {
            None
        }
    }
}

fn read(path: &Path) -> Result<String, ContentError> {
    fs::read_to_string(path).map_err(|source| ContentError::Read {
        path: path.to_path_buf(),
        source,
    })
}

fn parse<T: DeserializeOwned>(file: &str, source: &str) -> Result<T, ContentError> {
    ron::from_str(source).map_err(|source| ContentError::Parse {
        file: file.to_owned(),
        source: Box::new(source),
    })
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::Content;
    use crate::{ContentError, ItemId, TreeId};

    const ITEMS: &str = "{ \"oak_logs\": (stack_size: 1) }";
    const OAK: &str = "level: 1, log: \"oak_logs\", experience: 25, seconds_per_log: 3.0, logs: 3, regrow_seconds: 30.0";

    fn trees(oak: &str) -> String {
        format!("{{ \"oak\": ({oak}) }}")
    }

    fn error_with_oak(oak: &str) -> String {
        Content::from_sources(ITEMS, &trees(oak))
            .unwrap_err()
            .to_string()
    }

    #[test]
    fn the_repository_content_is_valid() {
        let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../content");

        let content = Content::load(&directory).unwrap();

        let oak = content.tree(&TreeId::new("oak")).unwrap();
        assert_eq!(oak.log, ItemId::new("oak_logs"));
        assert!(content.item(&oak.log).is_some());
    }

    #[test]
    fn valid_sources_give_their_definitions() {
        let content = Content::from_sources(ITEMS, &trees(OAK)).unwrap();

        assert_eq!(
            content.item(&ItemId::new("oak_logs")).unwrap().stack_size,
            1
        );
        assert_eq!(content.tree(&TreeId::new("oak")).unwrap().logs, 3);
        assert!(content.tree(&TreeId::new("ash")).is_none());
    }

    #[test]
    fn a_tree_with_an_unknown_log_is_an_error() {
        assert_eq!(
            error_with_oak(&OAK.replace("oak_logs", "ash_logs")),
            "the tree oak gives the unknown item ash_logs"
        );
    }

    #[test]
    fn a_tree_without_logs_is_an_error() {
        assert_eq!(
            error_with_oak(&OAK.replace("logs: 3", "logs: 0")),
            "the tree oak is not valid: it must give at least one log"
        );
    }

    #[test]
    fn a_tree_with_a_time_that_is_not_positive_is_an_error() {
        assert_eq!(
            error_with_oak(&OAK.replace("seconds_per_log: 3.0", "seconds_per_log: 0.0")),
            "the tree oak is not valid: its times must be more than zero"
        );
        assert_eq!(
            error_with_oak(&OAK.replace("regrow_seconds: 30.0", "regrow_seconds: -1.0")),
            "the tree oak is not valid: its times must be more than zero"
        );
    }

    #[test]
    fn a_tree_with_a_level_outside_1_to_99_is_an_error() {
        assert!(error_with_oak(&OAK.replace("level: 1", "level: 0")).contains("trees.ron"));
    }

    #[test]
    fn an_item_with_a_stack_size_of_zero_is_an_error() {
        let error = Content::from_sources("{ \"oak_logs\": (stack_size: 0) }", &trees(OAK))
            .unwrap_err()
            .to_string();

        assert_eq!(
            error,
            "the item oak_logs is not valid: the stack size must be at least 1"
        );
    }

    #[test]
    fn an_unknown_field_is_an_error() {
        let items = "{ \"oak_logs\": (stack_size: 1, weight: 2) }";

        assert!(matches!(
            Content::from_sources(items, &trees(OAK)),
            Err(ContentError::Parse { .. })
        ));
    }

    #[test]
    fn a_missing_directory_is_an_error() {
        assert!(matches!(
            Content::load(Path::new("missing-content-directory")),
            Err(ContentError::Read { .. })
        ));
    }
}
