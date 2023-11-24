#![allow(dead_code)]

use serde::Deserialize;

#[derive(Deserialize)]
pub struct QueueData {
    #[serde(rename = "requestId")] pub request_id: i32,
    #[serde(rename = "leaderboardInfo")] pub leaderbaord_info: MapData,
    #[serde(default)] pub difficulties: Vec<Difficulties>,
}
#[derive(Deserialize)]
pub struct MapData {
    #[serde(rename = "songName")] pub song_name: String,
    #[serde(rename = "songHash")] pub song_hash: String,
    #[serde(rename = "levelAuthorName")] pub level_author: String,
}
#[derive(Deserialize)]
pub struct Difficulties {
    pub difficulty: i32
}