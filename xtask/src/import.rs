mod blender;
mod clip_list;
mod frame_cuts;
mod recipe;
mod style100;

use std::fs;
use std::path::Path;

use anyhow::{Context, Result, ensure};
use serde::Serialize;

use blender::script_arguments;
pub use blender::{Blender, Program};
use clip_list::{BlenderClip, ClipList};
use recipe::HumanRecipe;
use style100::Style100;

use crate::naming::is_snake_case;

pub const HUMANS: &str = "art/humans";
pub const CLIPS: &str = "art/clips.ron";
pub const OUTPUT: &str = "assets/humans";
pub const WORK: &str = "target/import";
pub const SCRIPTS: &str = "xtask/blender";

const FIRST_BLENDER_FRAME: u32 = 1;
const SCRIPT_LINES: [&str; 3] = ["HUMAN ", "CLIP ", "EXPORTED "];

#[derive(Serialize)]
struct BlenderSpec<'a> {
    clips: Vec<BlenderClip<'a>>,
}

pub fn run(
    root: &Path,
    sources: &Path,
    blender: &impl Blender,
    report: &mut impl FnMut(String),
) -> Result<()> {
    report(format!("Blender {}", blender::check_version(blender)?));
    let recipes = read_recipes(&root.join(HUMANS))?;
    let text =
        fs::read_to_string(root.join(CLIPS)).with_context(|| format!("cannot read {CLIPS}"))?;
    let clips = ClipList::parse(&text).with_context(|| format!("{CLIPS} is not valid"))?;
    let work = root.join(WORK);
    let spec = work.join("clips.json");
    write_clip_spec(&clips, &sources.join(style100::ARCHIVE), &work, &spec)?;
    let humans = work.join("humans");
    let output = root.join(OUTPUT);
    for folder in [&humans, &output] {
        fs::create_dir_all(folder)
            .with_context(|| format!("cannot create {}", folder.display()))?;
    }
    let scripts = root.join(SCRIPTS);
    for (name, recipe) in &recipes {
        let recipe_path = humans.join(format!("{name}.json"));
        fs::write(&recipe_path, recipe.to_json()?)
            .with_context(|| format!("cannot write {}", recipe_path.display()))?;
        let body = humans.join(format!("{name}.blend"));
        report(format!("building the human {name}"));
        let lines = blender.run(&script_arguments(
            &scripts.join("human.py"),
            &[&recipe_path, &body],
        ))?;
        forward(&lines, report);
        report(format!("cutting the clips for {name}"));
        let glb = output.join(format!("{name}.glb"));
        let clip_report = output.join(format!("{name}.clips.json"));
        let lines = blender.run(&script_arguments(
            &scripts.join("clips.py"),
            &[&body, &spec, &glb, &clip_report],
        ))?;
        forward(&lines, report);
    }
    let noun = if recipes.len() == 1 {
        "human"
    } else {
        "humans"
    };
    report(format!("imported {} {noun} to {OUTPUT}", recipes.len()));
    Ok(())
}

fn read_recipes(folder: &Path) -> Result<Vec<(String, HumanRecipe)>> {
    let mut recipes = Vec::new();
    for entry in fs::read_dir(folder).with_context(|| format!("cannot read {HUMANS}"))? {
        let path = entry
            .with_context(|| format!("cannot read {HUMANS}"))?
            .path();
        if path.extension().is_none_or(|extension| extension != "ron") {
            continue;
        }
        let name = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .filter(|stem| is_snake_case(stem))
            .with_context(|| {
                format!(
                    "the recipe {} has a name that is not lowercase snake case",
                    path.display()
                )
            })?
            .to_owned();
        let text =
            fs::read_to_string(&path).with_context(|| format!("cannot read {}", path.display()))?;
        let recipe = HumanRecipe::parse(&text)
            .with_context(|| format!("the recipe {} is not valid", path.display()))?;
        recipes.push((name, recipe));
    }
    ensure!(!recipes.is_empty(), "{HUMANS} has no human recipes");
    recipes.sort_by(|left, right| left.0.cmp(&right.0));
    Ok(recipes)
}

fn write_clip_spec(clips: &ClipList, archive: &Path, work: &Path, spec: &Path) -> Result<()> {
    let mut style100 = Style100::open(archive)?;
    let cuts = style100.frame_cuts()?;
    let captures = work.join("100style");
    let mut entries = Vec::new();
    for clip in &clips.clips {
        let (start, stop) = cuts
            .range(&clip.style, clip.capture)
            .with_context(|| format!("the clip {} has no frame range", clip.name))?;
        let bvh = style100.extract(&clip.style, clip.capture, &captures)?;
        entries.push((
            clip,
            bvh,
            (start + FIRST_BLENDER_FRAME, stop + FIRST_BLENDER_FRAME),
        ));
    }
    let clips = entries
        .iter()
        .map(|(clip, bvh, frames)| clip.for_blender(bvh, *frames))
        .collect();
    let text = serde_json::to_string_pretty(&BlenderSpec { clips })
        .context("cannot write the clip list as JSON")?;
    fs::write(spec, text).with_context(|| format!("cannot write {}", spec.display()))
}

fn forward(lines: &str, report: &mut impl FnMut(String)) {
    for line in lines.lines() {
        if SCRIPT_LINES.iter().any(|prefix| line.starts_with(prefix)) {
            report(line.to_owned());
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::ffi::OsString;
    use std::fs;
    use std::path::{Path, PathBuf};

    use anyhow::{Result, bail};
    use serde_json::json;
    use tempfile::TempDir;

    use super::blender::script_arguments;
    use super::style100::ARCHIVE;
    use super::{Blender, CLIPS, HUMANS, OUTPUT, SCRIPTS, WORK, run};
    use crate::assets::testing::zip_with;

    const RECIPE: &str = r#"(shape: {"gender": 1.0}, skin: "Skin", eyes: "Eyes", eyebrows: "Brows", eyelashes: "Lashes", hair: "Hair", clothes: ["Suit"])"#;
    const CLIP_LIST: &str = r#"(clips: [(name: "walk", style: "Neutral", capture: ForwardWalk, cut: Cycle(min_seconds: 0.8, max_seconds: 1.6, max_turn_degrees: 4.0))])"#;
    const CUTS: &str = "STYLE_NAME,FW_START,FW_STOP\nNeutral,327,7445\n";
    const BLENDER_5_3: &str = "Blender 5.3.0\n\tbuild date: 2026-11-10\n";

    struct FakeBlender {
        version: &'static str,
        calls: RefCell<Vec<Vec<OsString>>>,
    }

    impl FakeBlender {
        fn new(version: &'static str) -> Self {
            Self {
                version,
                calls: RefCell::new(Vec::new()),
            }
        }

        fn calls(&self) -> Vec<Vec<OsString>> {
            self.calls.borrow().clone()
        }
    }

    impl Blender for FakeBlender {
        fn run(&self, arguments: &[OsString]) -> Result<String> {
            self.calls.borrow_mut().push(arguments.to_vec());
            if arguments == [OsString::from("--version")] {
                return Ok(self.version.to_owned());
            }
            let position = |wanted: &str| arguments.iter().position(|argument| argument == wanted);
            let (Some(python), Some(dashes)) = (position("--python"), position("--")) else {
                bail!("the test does not know the arguments {arguments:?}");
            };
            let script = PathBuf::from(&arguments[python + 1]);
            let after = &arguments[dashes + 1..];
            if script.ends_with("human.py") {
                fs::write(&after[1], "blend")?;
                return Ok(
                    "Blender noise\nHUMAN height 1.729 m, triangles 40376, bones 53\n".to_owned(),
                );
            }
            if script.ends_with("clips.py") {
                fs::write(&after[2], "glb")?;
                fs::write(&after[3], "{}")?;
                return Ok("CLIP walk: 1.37 s, 0.81 m/s\nEXPORTED hero.glb\n".to_owned());
            }
            bail!("the test does not know the script {}", script.display())
        }
    }

    fn setup() -> (TempDir, PathBuf, PathBuf) {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().join("root");
        fs::create_dir_all(root.join(HUMANS)).unwrap();
        fs::write(root.join(HUMANS).join("hero.ron"), RECIPE).unwrap();
        fs::write(root.join(CLIPS), CLIP_LIST).unwrap();
        let sources = directory.path().join("sources");
        let archive = sources.join(ARCHIVE);
        let folder = archive.parent().unwrap();
        fs::create_dir_all(folder).unwrap();
        zip_with(
            folder,
            "100STYLE.zip",
            &[
                ("100STYLE/Frame_Cuts.csv", CUTS),
                ("100STYLE/Neutral/Neutral_FW.bvh", "HIERARCHY"),
            ],
        );
        (directory, root, sources)
    }

    fn import(root: &Path, sources: &Path, blender: &FakeBlender) -> Result<Vec<String>> {
        let mut lines = Vec::new();
        run(root, sources, blender, &mut |line| lines.push(line))?;
        Ok(lines)
    }

    fn json_in(path: &Path) -> serde_json::Value {
        serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap()
    }

    #[test]
    fn import_checks_blender_then_builds_the_human_and_cuts_its_clips() {
        let (_directory, root, sources) = setup();
        let blender = FakeBlender::new(BLENDER_5_3);

        import(&root, &sources, &blender).unwrap();

        let work = root.join(WORK);
        let output = root.join(OUTPUT);
        let recipe = work.join("humans").join("hero.json");
        let body = work.join("humans").join("hero.blend");
        let spec = work.join("clips.json");
        let glb = output.join("hero.glb");
        let clips = output.join("hero.clips.json");
        assert_eq!(
            blender.calls(),
            [
                vec![OsString::from("--version")],
                script_arguments(&root.join(SCRIPTS).join("human.py"), &[&recipe, &body]),
                script_arguments(
                    &root.join(SCRIPTS).join("clips.py"),
                    &[&body, &spec, &glb, &clips]
                ),
            ]
        );
        assert_eq!(fs::read_to_string(glb).unwrap(), "glb");
    }

    #[test]
    fn the_recipe_and_the_clip_list_go_to_blender_as_json_files() {
        let (_directory, root, sources) = setup();

        import(&root, &sources, &FakeBlender::new(BLENDER_5_3)).unwrap();

        let work = root.join(WORK);
        let bvh = work.join("100style").join("Neutral_FW.bvh");
        assert_eq!(
            json_in(&work.join("humans").join("hero.json"))["hair"],
            "Hair"
        );
        assert_eq!(
            json_in(&work.join("clips.json")),
            json!({
                "clips": [{
                    "name": "walk",
                    "kind": "cycle",
                    "bvh": bvh,
                    "first": 328,
                    "last": 7446,
                    "min_seconds": 0.8,
                    "max_seconds": 1.6,
                    "max_turn_degrees": 4.0,
                }]
            })
        );
        assert_eq!(fs::read_to_string(bvh).unwrap(), "HIERARCHY");
    }

    #[test]
    fn the_report_shows_the_lines_of_the_scripts_but_not_other_blender_output() {
        let (_directory, root, sources) = setup();

        let lines = import(&root, &sources, &FakeBlender::new(BLENDER_5_3)).unwrap();

        assert_eq!(
            lines,
            [
                "Blender 5.3.0",
                "building the human hero",
                "HUMAN height 1.729 m, triangles 40376, bones 53",
                "cutting the clips for hero",
                "CLIP walk: 1.37 s, 0.81 m/s",
                "EXPORTED hero.glb",
                "imported 1 human to assets/humans",
            ]
        );
    }

    #[test]
    fn a_different_blender_version_stops_the_import_before_the_scripts() {
        let (_directory, root, sources) = setup();
        let blender = FakeBlender::new("Blender 4.2.0\n");

        assert!(import(&root, &sources, &blender).is_err());
        assert_eq!(blender.calls().len(), 1);
    }

    #[test]
    fn a_missing_100style_archive_is_an_error_with_its_path() {
        let (_directory, root, sources) = setup();
        fs::remove_file(sources.join(ARCHIVE)).unwrap();

        let error = import(&root, &sources, &FakeBlender::new(BLENDER_5_3)).unwrap_err();

        assert!(format!("{error:#}").contains("100STYLE.zip"));
    }

    #[test]
    fn a_folder_without_recipes_is_an_error() {
        let (_directory, root, sources) = setup();
        fs::remove_file(root.join(HUMANS).join("hero.ron")).unwrap();

        let error = import(&root, &sources, &FakeBlender::new(BLENDER_5_3)).unwrap_err();

        assert!(format!("{error:#}").contains(HUMANS));
    }
}
