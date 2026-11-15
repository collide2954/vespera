use anyhow::{Context, Result, bail, ensure};

use super::clip_list::Capture;

const STYLE_COLUMN: &str = "STYLE_NAME";
const NO_FRAME: &str = "N/A";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrameCuts {
    columns: Vec<String>,
    rows: Vec<Vec<String>>,
}

impl FrameCuts {
    pub fn parse(text: &str) -> Result<Self> {
        let text = text.strip_prefix('\u{feff}').unwrap_or(text);
        let mut lines = text.lines().filter(|line| !line.trim().is_empty());
        let columns = cells(lines.next().context("Frame_Cuts.csv is empty")?);
        ensure!(
            columns.first().map(String::as_str) == Some(STYLE_COLUMN),
            "Frame_Cuts.csv does not start with the column {STYLE_COLUMN}"
        );
        let mut rows = Vec::new();
        for (index, line) in lines.enumerate() {
            let row = cells(line);
            ensure!(
                row.len() == columns.len(),
                "row {} of Frame_Cuts.csv has {} cells, but the header has {} cells",
                index + 1,
                row.len(),
                columns.len()
            );
            rows.push(row);
        }
        Ok(Self { columns, rows })
    }

    pub fn range(&self, style: &str, capture: Capture) -> Result<(u32, u32)> {
        let row = self
            .rows
            .iter()
            .find(|row| row[0] == style)
            .with_context(|| format!("Frame_Cuts.csv has no style with the name {style}"))?;
        let code = capture.code();
        let start = self.frame(row, &format!("{code}_START"))?;
        let stop = self.frame(row, &format!("{code}_STOP"))?;
        let (Some(start), Some(stop)) = (start, stop) else {
            bail!("100STYLE has no {code} capture for the style {style}");
        };
        ensure!(
            start < stop,
            "the {code} capture of {style} starts at frame {start}, which is not before its stop at frame {stop}"
        );
        Ok((start, stop))
    }

    fn frame(&self, row: &[String], column: &str) -> Result<Option<u32>> {
        let index = self
            .columns
            .iter()
            .position(|name| name == column)
            .with_context(|| format!("Frame_Cuts.csv has no column {column}"))?;
        let cell = &row[index];
        if cell == NO_FRAME {
            return Ok(None);
        }
        cell.parse().map(Some).with_context(|| {
            format!("the column {column} of Frame_Cuts.csv has the value {cell}, which is not a frame number")
        })
    }
}

fn cells(line: &str) -> Vec<String> {
    line.split(',').map(|cell| cell.trim().to_owned()).collect()
}

#[cfg(test)]
mod tests {
    use super::FrameCuts;
    use crate::import::clip_list::Capture;

    const CUTS: &str = "STYLE_NAME,FR_START,FR_STOP,FW_START,FW_STOP,ID_START,ID_STOP\r\nAkimbo,347,4067,384,7204,461,1178\r\nNeutral,357,4227,327,7445,N/A,N/A\r\n";

    fn error(cuts: &FrameCuts, style: &str, capture: Capture) -> String {
        format!("{:#}", cuts.range(style, capture).unwrap_err())
    }

    #[test]
    fn a_range_comes_from_the_start_and_stop_columns_of_the_capture() {
        let cuts = FrameCuts::parse(CUTS).unwrap();

        assert_eq!(
            cuts.range("Neutral", Capture::ForwardWalk).unwrap(),
            (327, 7445)
        );
        assert_eq!(
            cuts.range("Neutral", Capture::ForwardRun).unwrap(),
            (357, 4227)
        );
        assert_eq!(cuts.range("Akimbo", Capture::Idle).unwrap(), (461, 1178));
    }

    #[test]
    fn a_byte_order_mark_is_ignored() {
        let cuts = FrameCuts::parse(&format!("\u{feff}{CUTS}")).unwrap();

        assert_eq!(
            cuts.range("Akimbo", Capture::ForwardRun).unwrap(),
            (347, 4067)
        );
    }

    #[test]
    fn an_unknown_style_is_an_error_with_its_name() {
        let cuts = FrameCuts::parse(CUTS).unwrap();

        assert!(error(&cuts, "Zombie", Capture::ForwardWalk).contains("Zombie"));
    }

    #[test]
    fn a_capture_without_a_range_is_an_error() {
        let cuts = FrameCuts::parse(CUTS).unwrap();

        let message = error(&cuts, "Neutral", Capture::Idle);

        assert!(message.contains("ID"), "{message}");
        assert!(message.contains("Neutral"), "{message}");
    }

    #[test]
    fn a_capture_without_columns_is_an_error() {
        let cuts = FrameCuts::parse(CUTS).unwrap();

        assert!(error(&cuts, "Neutral", Capture::SidestepRun).contains("SR_START"));
    }

    #[test]
    fn a_start_after_the_stop_is_an_error() {
        let cuts = FrameCuts::parse(&CUTS.replace("327,7445", "7445,327")).unwrap();

        assert!(error(&cuts, "Neutral", Capture::ForwardWalk).contains("7445"));
    }

    #[test]
    fn a_row_with_a_different_cell_count_is_an_error() {
        assert!(FrameCuts::parse(&CUTS.replace(",461,1178", ",461")).is_err());
    }

    #[test]
    fn a_table_without_the_style_column_is_an_error() {
        assert!(FrameCuts::parse(&CUTS.replace("STYLE_NAME", "NAME")).is_err());
    }
}
