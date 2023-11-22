use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Tile {
    Black,
    Yellow,
    Green,
}

impl TryFrom<String> for Tile {
    type Error = ();

    fn try_from(value: String) -> Result<Self, Self::Error> {
        use Tile::*;

        Ok(match value.as_str() {
            "black_large_square" => Black,
            "large_yellow_square" => Yellow,
            "large_green_square" => Green,
            "⬛" => Black,
            "🟨" => Yellow,
            "🟩" => Green,
            _ => {
                return Err(());
            }
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Puzzle {
    pub day_offset: i32,
    pub attempts: i32,
    pub solved: bool,
    pub hard_mode: bool,
    pub board: Vec<Vec<Tile>>,
}

impl TryFrom<String> for Puzzle {
    type Error = ();

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let mut lines = value.lines();
        let first_line = lines.next().unwrap_or("");

        let re = Regex::new(r"Wordle (?<day_offset>\d+) (?<attempts>(\d|X))\/6(?<hard_mode>\*)?")
            .unwrap();
        let Some(captures) = re.captures(first_line) else {
            return Err(());
        };

        let day_offset: i32 = captures["day_offset"].parse().unwrap();
        let attempts: i32 = captures["attempts"].parse().unwrap_or(6);
        let solved = match &captures["attempts"] {
            "X" => false,
            _ => true,
        };
        let hard_mode = captures.name("hard_mode").is_some();
        let mut board: Vec<Vec<Tile>> = Vec::new();

        while let Some(line) = lines.next() {
            if board.len() == 6 {
                break;
            }

            if line.is_empty() {
                continue;
            }

            // @TODO there's currently no constaints on how long a row is, but
            // we should ensure there is exactly five for it to be valid.

            let row: Vec<Tile> = if line.contains("::") {
                // Handle Slack messages which convert emoji to textual representation.
                line.split("::")
                    .into_iter()
                    .map(|t| {
                        let t = t.replace(":", "");
                        t.try_into().unwrap()
                    })
                    .collect()
            } else {
                // Handle Discord messages which just use raw emoji.
                line.split("")
                    .filter(|&x| !x.is_empty())
                    .into_iter()
                    .map(|t| t.to_string().try_into().unwrap())
                    .collect()
            };

            board.push(row);
        }

        // @TODO technically the board should have _at least_ one row, and if
        // that row is not completely green it is not valid either.

        Ok(Puzzle {
            day_offset,
            attempts,
            solved,
            hard_mode,
            board,
        })
    }
}
