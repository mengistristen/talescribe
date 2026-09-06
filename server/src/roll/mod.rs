use std::{fs::File, io::Read, path::PathBuf};

use serde::{Deserialize, Serialize};

pub mod parser;

const TABLES_PATH: &str = "./.talescribe/tables/";

#[derive(Debug, PartialEq)]
pub enum Roll {
    Dice(DiceRoll),
    Table(TableRoll),
}

#[derive(Debug, PartialEq)]
pub struct DiceRoll {
    die: u16,
    count: usize,
    modifier: u16,
}

#[derive(Debug, PartialEq)]
pub struct TableRoll {
    name: String,
}

#[derive(Serialize, Deserialize)]
struct Table {
    options: Vec<String>,
}

impl DiceRoll {
    fn evaluate(self) -> u16 {
        std::iter::repeat_with(|| rand::random_range(1..=self.die))
            .take(self.count)
            .reduce(|accum, val| accum + val)
            .unwrap_or(0)
            + self.modifier
    }
}

impl TableRoll {
    fn evaluate(self) -> Option<String> {
        let mut path = PathBuf::from(TABLES_PATH);

        path.push(format!("{}.toml", self.name));

        let mut file = File::open(path).ok()?;
        let mut contents = String::new();

        file.read_to_string(&mut contents).ok()?;

        let table: Table = toml::from_str(&contents).ok()?;

        Some(table.options[rand::random_range(..table.options.len())].clone())
    }
}

impl Roll {
    pub fn evaluate(self) -> Option<String> {
        match self {
            Roll::Dice(dice_roll) => Some(dice_roll.evaluate().to_string()),
            Roll::Table(table_roll) => table_roll.evaluate(),
        }
    }
}
