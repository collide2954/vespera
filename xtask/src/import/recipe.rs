use std::collections::BTreeMap;

use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct HumanRecipe {
    pub shape: BTreeMap<String, f64>,
    pub skin: String,
    pub eyes: String,
    pub eyebrows: String,
    pub eyelashes: String,
    pub hair: String,
    pub clothes: Vec<String>,
}

impl HumanRecipe {
    pub fn parse(text: &str) -> Result<Self> {
        let recipe: Self = ron::from_str(text).context("the human recipe has a bad format")?;
        for (name, value) in &recipe.shape {
            ensure!(
                (0.0..=1.0).contains(value),
                "the shape value {name} is {value}. Shape values go from 0 to 1"
            );
        }
        for (part, asset) in [
            ("skin", &recipe.skin),
            ("eyes", &recipe.eyes),
            ("eyebrows", &recipe.eyebrows),
            ("eyelashes", &recipe.eyelashes),
            ("hair", &recipe.hair),
        ] {
            ensure!(
                !asset.trim().is_empty(),
                "the recipe has no name for the {part} asset"
            );
        }
        ensure!(
            recipe
                .clothes
                .iter()
                .all(|garment| !garment.trim().is_empty()),
            "the recipe has a garment without a name"
        );
        Ok(recipe)
    }

    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(self).context("cannot write the recipe as JSON")
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::fs;

    use serde_json::json;

    use super::HumanRecipe;
    use crate::import::HUMANS;
    use crate::repository;

    const RECIPE: &str = r#"(
        shape: { "gender": 1.0, "height": 0.5 },
        skin: "Young caucasian male",
        eyes: "Low-poly",
        eyebrows: "Eyebrow001",
        eyelashes: "Eyelashes01",
        hair: "Short02",
        clothes: ["Male casualsuit01", "Shoes03"],
    )"#;

    fn error(text: &str) -> String {
        format!("{:#}", HumanRecipe::parse(text).unwrap_err())
    }

    #[test]
    fn a_recipe_gives_the_shape_and_the_assets() {
        assert_eq!(
            HumanRecipe::parse(RECIPE).unwrap(),
            HumanRecipe {
                shape: BTreeMap::from([("gender".to_owned(), 1.0), ("height".to_owned(), 0.5)]),
                skin: "Young caucasian male".to_owned(),
                eyes: "Low-poly".to_owned(),
                eyebrows: "Eyebrow001".to_owned(),
                eyelashes: "Eyelashes01".to_owned(),
                hair: "Short02".to_owned(),
                clothes: vec!["Male casualsuit01".to_owned(), "Shoes03".to_owned()],
            }
        );
    }

    #[test]
    fn the_json_for_blender_has_the_same_values() {
        let text = HumanRecipe::parse(RECIPE).unwrap().to_json().unwrap();

        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&text).unwrap(),
            json!({
                "shape": { "gender": 1.0, "height": 0.5 },
                "skin": "Young caucasian male",
                "eyes": "Low-poly",
                "eyebrows": "Eyebrow001",
                "eyelashes": "Eyelashes01",
                "hair": "Short02",
                "clothes": ["Male casualsuit01", "Shoes03"],
            })
        );
    }

    #[test]
    fn a_shape_value_outside_0_to_1_is_an_error() {
        let message = error(&RECIPE.replace("\"height\": 0.5", "\"height\": 1.5"));

        assert!(message.contains("height"), "{message}");
        assert!(message.contains("1.5"), "{message}");
    }

    #[test]
    fn an_empty_asset_name_is_an_error() {
        let message = error(&RECIPE.replace("\"Short02\"", "\"\""));

        assert!(message.contains("hair"), "{message}");
    }

    #[test]
    fn an_empty_garment_name_is_an_error() {
        let message = error(&RECIPE.replace("\"Shoes03\"", "\" \""));

        assert!(message.contains("garment"), "{message}");
    }

    #[test]
    fn an_unknown_field_is_an_error() {
        let text = RECIPE.replace("hair:", "beard: \"Full\",\n        hair:");

        assert!(HumanRecipe::parse(&text).is_err());
    }

    #[test]
    fn the_repository_recipes_are_valid() {
        let folder = repository::root().join(HUMANS);
        let mut names = Vec::new();
        for entry in fs::read_dir(&folder).unwrap() {
            let path = entry.unwrap().path();
            let text = fs::read_to_string(&path).unwrap();
            assert!(
                HumanRecipe::parse(&text).is_ok(),
                "{} is not valid",
                path.display()
            );
            names.push(path.file_name().unwrap().to_string_lossy().into_owned());
        }
        names.sort();

        assert_eq!(names, ["player_female.ron", "player_male.ron"]);
    }
}
