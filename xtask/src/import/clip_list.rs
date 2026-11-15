use std::collections::HashSet;
use std::path::Path;

use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};

use crate::naming::is_snake_case;

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClipList {
    pub clips: Vec<Clip>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Clip {
    pub name: String,
    pub style: String,
    pub capture: Capture,
    pub cut: Cut,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum Capture {
    Idle,
    ForwardWalk,
    ForwardRun,
    BackwardWalk,
    BackwardRun,
    SidestepWalk,
    SidestepRun,
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Cut {
    Idle {
        min_seconds: f64,
        max_seconds: f64,
        max_travel_meters: f64,
    },
    Cycle {
        min_seconds: f64,
        max_seconds: f64,
        max_turn_degrees: f64,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct BlenderClip<'a> {
    name: &'a str,
    kind: &'static str,
    bvh: &'a Path,
    first: u32,
    last: u32,
    min_seconds: f64,
    max_seconds: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_travel_meters: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_turn_degrees: Option<f64>,
}

impl ClipList {
    pub fn parse(text: &str) -> Result<Self> {
        let list: Self = ron::from_str(text).context("the clip list has a bad format")?;
        ensure!(!list.clips.is_empty(), "the clip list has no clips");
        let mut names = HashSet::new();
        for clip in &list.clips {
            clip.validate()?;
            ensure!(
                names.insert(clip.name.as_str()),
                "the clip list has the clip {} two times",
                clip.name
            );
        }
        Ok(list)
    }
}

impl Clip {
    pub fn for_blender<'a>(&'a self, bvh: &'a Path, frames: (u32, u32)) -> BlenderClip<'a> {
        let (kind, min_seconds, max_seconds, max_travel_meters, max_turn_degrees) = match self.cut {
            Cut::Idle {
                min_seconds,
                max_seconds,
                max_travel_meters,
            } => (
                "idle",
                min_seconds,
                max_seconds,
                Some(max_travel_meters),
                None,
            ),
            Cut::Cycle {
                min_seconds,
                max_seconds,
                max_turn_degrees,
            } => (
                "cycle",
                min_seconds,
                max_seconds,
                None,
                Some(max_turn_degrees),
            ),
        };
        BlenderClip {
            name: &self.name,
            kind,
            bvh,
            first: frames.0,
            last: frames.1,
            min_seconds,
            max_seconds,
            max_travel_meters,
            max_turn_degrees,
        }
    }

    fn validate(&self) -> Result<()> {
        let name = &self.name;
        ensure!(
            is_snake_case(name),
            "the clip name {name:?} is not lowercase snake case"
        );
        ensure!(
            !self.style.is_empty() && self.style.bytes().all(|byte| byte.is_ascii_alphanumeric()),
            "the clip {name} has the style {:?}. A style name has only letters and digits",
            self.style
        );
        self.cut
            .validate()
            .with_context(|| format!("the cut rules of the clip {name} are not valid"))
    }
}

impl Capture {
    pub fn code(self) -> &'static str {
        match self {
            Self::Idle => "ID",
            Self::ForwardWalk => "FW",
            Self::ForwardRun => "FR",
            Self::BackwardWalk => "BW",
            Self::BackwardRun => "BR",
            Self::SidestepWalk => "SW",
            Self::SidestepRun => "SR",
        }
    }
}

impl Cut {
    fn validate(self) -> Result<()> {
        let (min_seconds, max_seconds, limit) = match self {
            Self::Idle {
                min_seconds,
                max_seconds,
                max_travel_meters,
            } => (min_seconds, max_seconds, max_travel_meters),
            Self::Cycle {
                min_seconds,
                max_seconds,
                max_turn_degrees,
            } => (min_seconds, max_seconds, max_turn_degrees),
        };
        ensure!(
            min_seconds > 0.0,
            "the minimum time is {min_seconds} s, but it must be more than 0 s"
        );
        ensure!(
            max_seconds >= min_seconds,
            "the maximum time {max_seconds} s is less than the minimum time {min_seconds} s"
        );
        ensure!(limit >= 0.0, "the limit {limit} is less than 0");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::{Capture, Clip, ClipList, Cut};
    use crate::import::CLIPS;
    use crate::repository;

    const LIST: &str = r#"(
        clips: [
            (
                name: "idle",
                style: "Neutral",
                capture: Idle,
                cut: Idle(min_seconds: 2.0, max_seconds: 4.0, max_travel_meters: 0.15),
            ),
            (
                name: "walk",
                style: "Neutral",
                capture: ForwardWalk,
                cut: Cycle(min_seconds: 0.8, max_seconds: 1.6, max_turn_degrees: 4.0),
            ),
        ],
    )"#;

    fn error(text: &str) -> String {
        format!("{:#}", ClipList::parse(text).unwrap_err())
    }

    #[test]
    fn a_clip_list_gives_the_capture_and_the_cut_rules_of_each_clip() {
        assert_eq!(
            ClipList::parse(LIST).unwrap(),
            ClipList {
                clips: vec![
                    Clip {
                        name: "idle".to_owned(),
                        style: "Neutral".to_owned(),
                        capture: Capture::Idle,
                        cut: Cut::Idle {
                            min_seconds: 2.0,
                            max_seconds: 4.0,
                            max_travel_meters: 0.15,
                        },
                    },
                    Clip {
                        name: "walk".to_owned(),
                        style: "Neutral".to_owned(),
                        capture: Capture::ForwardWalk,
                        cut: Cut::Cycle {
                            min_seconds: 0.8,
                            max_seconds: 1.6,
                            max_turn_degrees: 4.0,
                        },
                    },
                ],
            }
        );
    }

    #[test]
    fn each_capture_has_its_100style_code() {
        let codes = [
            Capture::Idle,
            Capture::ForwardWalk,
            Capture::ForwardRun,
            Capture::BackwardWalk,
            Capture::BackwardRun,
            Capture::SidestepWalk,
            Capture::SidestepRun,
        ]
        .map(Capture::code);

        assert_eq!(codes, ["ID", "FW", "FR", "BW", "BR", "SW", "SR"]);
    }

    #[test]
    fn a_clip_name_needs_lowercase_snake_case() {
        for name in ["Walk", "walk-fast", "walk__fast", ""] {
            let text = LIST.replace("\"walk\"", &format!("\"{name}\""));

            assert!(
                error(&text).contains("snake case"),
                "the name {name:?} must be an error"
            );
        }
    }

    #[test]
    fn two_clips_cannot_have_the_same_name() {
        let message = error(&LIST.replace("\"walk\"", "\"idle\""));

        assert!(message.contains("two times"), "{message}");
    }

    #[test]
    fn an_empty_style_is_an_error() {
        let message = error(&LIST.replacen("\"Neutral\"", "\"\"", 1));

        assert!(message.contains("style"), "{message}");
    }

    #[test]
    fn a_maximum_time_below_the_minimum_time_is_an_error() {
        let message = error(&LIST.replace("max_seconds: 1.6", "max_seconds: 0.5"));

        assert!(message.contains("walk"), "{message}");
        assert!(message.contains("0.5"), "{message}");
    }

    #[test]
    fn a_minimum_time_of_zero_is_an_error() {
        let message = error(&LIST.replace("min_seconds: 0.8", "min_seconds: 0.0"));

        assert!(message.contains("walk"), "{message}");
    }

    #[test]
    fn a_negative_limit_is_an_error() {
        assert!(
            error(&LIST.replace("max_turn_degrees: 4.0", "max_turn_degrees: -1.0"))
                .contains("walk")
        );
        assert!(
            error(&LIST.replace("max_travel_meters: 0.15", "max_travel_meters: -0.1"))
                .contains("idle")
        );
    }

    #[test]
    fn an_unknown_cut_rule_is_an_error() {
        let text = LIST.replace("max_turn_degrees: 4.0", "max_turn_degrees: 4.0, speed: 2.0");

        assert!(ClipList::parse(&text).is_err());
    }

    #[test]
    fn a_list_without_clips_is_an_error() {
        assert!(error("(clips: [])").contains("no clips"));
    }

    #[test]
    fn the_repository_clip_list_has_idle_walk_and_jog() {
        let text = fs::read_to_string(repository::root().join(CLIPS)).unwrap();

        let names: Vec<String> = ClipList::parse(&text)
            .unwrap()
            .clips
            .into_iter()
            .map(|clip| clip.name)
            .collect();

        assert_eq!(names, ["idle", "walk", "jog"]);
    }
}
