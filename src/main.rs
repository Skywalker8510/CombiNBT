#![allow(dead_code)]

use fastnbt::from_bytes;
use flate2::read::GzDecoder;
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use flate2::Compression;
use flate2::write::GzEncoder;

#[derive(Deserialize, Debug, Clone, Serialize)]
#[serde(rename_all = "PascalCase")]
struct ScoreboardRoot {
    data_version: i32,

    #[serde(rename = "data")]
    data: ScoreboardData,
}

#[derive(Deserialize, Debug, Default, Clone, Serialize)]
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

#[derive(Deserialize, Debug, Default, Clone, Serialize)]
#[serde(default)]
struct DisplaySlot {
    #[serde(rename = "list", default)]
    list: String
}

#[derive(Deserialize, Debug, Default, Clone, Serialize)]
#[serde(rename_all = "PascalCase")]
struct Objectives {
    criteria_name: Option<String>,
    display_name: String,
    name: String,
}

#[derive(Deserialize, Debug, Clone, Serialize)]
#[serde(rename_all = "PascalCase")]
struct PlayerScore {
    objective: String,
    name: String,
    score: Option<i32>,
    locked: Option<i8>,
}

#[derive(Deserialize, Debug, Default, Clone, Serialize)]
#[serde(rename_all = "PascalCase")]
struct Teams {
    display_name: String,
    name: String,
    players: Vec<String>,
    team_color: String,
}

#[derive(Debug, thiserror::Error)]
enum ArgError {
    #[error("the command was empty")]
    InvalidArgument,
    #[error("the name inputted was not found in the Scoreboard")]
    NameNotFound,
}

fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let file = std::fs::File::open(args[0].clone()).unwrap();
    let old_name;
    if let Some(o) = args.get(1) {
        old_name = o
    } else {
        panic!("{:?}", ArgError::InvalidArgument);
    }
    let new_name;
    if let Some(n) = args.get(2) {
        new_name = n
    } else {
        panic!("{:?}", ArgError::InvalidArgument);
    }

    // Player dat files are compressed with GZip.
    let mut decoder = GzDecoder::new(file);
    let mut data = Vec::new();
    decoder.read_to_end(&mut data).expect("failed to read data");

    let mut scoreboard: ScoreboardRoot = from_bytes(&data).expect("failed to parse data");

    // println!("{:?}", scoreboard);

    let old_score_data = match scoreboard_for_player(scoreboard.data.player_scores.clone(), old_name.clone()) {
        Ok(data) => data,
        Err(e) => panic!("{} {}", e, old_name),
    };
    // println!("{:?}", old_score_data);
    let new_score_data = scoreboard_for_player(scoreboard.data.player_scores.clone(), new_name.clone()).unwrap_or_else(|e| {
        println!("{} {}", e, new_name);
        println!("Continuing run. will replace all instances of {} with {}", old_name, new_name);
        println!("Combining of data will not take place");
        Vec::new()
    });

    let mut updated_score_data = Vec::new();
    for data in &old_score_data {
        if let Some(found_data) = new_score_data.iter().find(|&new_data| data.objective == new_data.objective) {

            #[allow(clippy::manual_map)]
            let updated_score = if let Some(t) = data.score && let Some(y) = found_data.score{
                Some(t + y)
            } else if let Some(t) = data.score {
                Some(t)
            } else if let Some(y) = found_data.score {
                Some(y)
            } else {
                None
            };

            let updated_player_score = PlayerScore {
                objective: data.objective.clone(),
                name: new_name.clone(),
                score: updated_score,
                locked: found_data.locked,
            };
            updated_score_data.push(updated_player_score);
        } else {
            let mut updated_player_score = data.clone();
            updated_player_score.name = new_name.clone();
            updated_score_data.push(updated_player_score);
        }
    }
    for data in &new_score_data {
        if let Some(_found_data) = updated_score_data.iter().find(|&new_data| data.objective == new_data.objective) {
            continue;
        } else {
            let updated_player_score = data.clone();
            updated_score_data.push(updated_player_score);
        }
    }

    // println!("{:?}", updated_score_data);

    let player_score_no_old_name = scoreboard_excluding_player(scoreboard.data.player_scores, old_name.clone());
    let mut player_score_without_player;
    if !new_score_data.is_empty() {
        player_score_without_player = scoreboard_excluding_player(player_score_no_old_name, new_name.clone());
    } else {
        player_score_without_player = player_score_no_old_name;
    }

    player_score_without_player.extend_from_slice(&updated_score_data);

    scoreboard.data.player_scores = player_score_without_player;

    let new_bytes = fastnbt::to_bytes(&scoreboard).unwrap();
    let outfile = std::fs::File::create("scoreboard.dat").unwrap();
    let mut encoder = GzEncoder::new(outfile, Compression::fast());
    encoder.write_all(&new_bytes).unwrap();
}

fn scoreboard_for_player(player_score: Vec<PlayerScore>, player_name: String) -> Result<Vec<PlayerScore>, ArgError> {
    let output: Vec<PlayerScore> = player_score.into_iter().filter(|ps| ps.name == player_name).collect();
    if output.is_empty() {
        return Err(ArgError::NameNotFound)
    }
    Ok(output)
}

fn scoreboard_excluding_player(player_score: Vec<PlayerScore>, player_name: String) -> Vec<PlayerScore> {
    player_score.into_iter().filter(|ps| ps.name != player_name).collect()
}