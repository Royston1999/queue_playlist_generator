#![allow(dead_code)]

use serde::Serialize;

#[derive(Serialize)]
pub struct Playlist {
    #[serde(rename = "playlistTitle")] pub name: String,
    #[serde(rename = "playlistAuthor")] pub author: String,
    #[serde(rename = "playlistDescription")] pub description: String,
    pub image: String,
    pub songs: Vec<Song>
}
#[derive(Serialize)]
pub struct Song {
    #[serde(rename = "songName")] pub song_name: String,
    #[serde(rename = "levelAuthorName")] pub level_author: String,
    #[serde(rename = "hash")] pub song_hash: String,
    #[serde(skip_serializing_if = "Vec::is_empty")] pub difficulties: Vec<Difficulty>
}
#[derive(Serialize)]
pub struct Difficulty {
    pub characteristic: String,
    pub name: String
}