#![allow(dead_code)]

use fastnbt::from_bytes;
use flate2::read::GzDecoder;
use serde::Deserialize;
use std::io::Read;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct ScoreboardRoot {
    data_version: i32,

    #[serde(rename = "data")]
    data: ScoreboardData,
}

#[derive(Deserialize, Debug, Default)]
#[serde(default)]
struct ScoreboardData {
    #[serde(rename = "DisplaySlots", default)]
    display_slots: DisplaySlot,
    #[serde(rename = "Objectives", default)]
    objectives: Vec<Objectives>,
    #[serde(rename = "PlayerScores", default)]
    player_scores: Vec<PlayerScore>,
    #[serde(rename = "Teams", default)]
    teams: Vec<Teams>
}

#[derive(Deserialize, Debug, Default)]
#[serde(default)]
struct DisplaySlot {
    #[serde(rename = "list", default)]
    list: String
}

#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "PascalCase")]
struct Objectives {
    criteria_name: Option<String>,
    display_name: String,
    name: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct PlayerScore {
    objective: String,
    name: String,
    score: Option<i32>,
    locked: Option<i8>,
}

#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "PascalCase")]
struct Teams {
    display_name: String,
    name: String,
    players: Vec<String>,
    team_color: String,
}

fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let file = std::fs::File::open(args[0].clone()).unwrap();
    let old_name = args[1].clone();
    let new_name = args[2].clone();

    // Player dat files are compressed with GZip.
    let mut decoder = GzDecoder::new(file);
    let mut data = Vec::new();
    decoder.read_to_end(&mut data).expect("failed to read data");

    let scoreboard: ScoreboardRoot = from_bytes(&data).expect("failed to parse data");

    let old_score_data = scoreboard_for_player(scoreboard.data.player_scores, old_name.clone());
    println!("{:?}", old_score_data);
}

fn scoreboard_for_player(player_score: Vec<PlayerScore>, player_name: String) -> Vec<PlayerScore> {
    player_score.into_iter().filter(|ps| ps.name == player_name).collect()
}