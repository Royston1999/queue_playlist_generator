use std::{error::Error, io::{self, BufReader, Read}, fs::File, sync::{Arc, Mutex}};
use base64::{engine::general_purpose, Engine};
use queue_playlist_maker::lock;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use serde::{Deserialize, Serialize};

use crate::{AppData, GenStatus};

use super::playlist_data::{Playlist, Song, Difficulty};
use super::api_data::{QueueData, Difficulties};

static SS_URL: &str = "https://scoresaber.com/api/ranking/requests/";
static SS_MAP_URL: &str = "https://scoresaber.com/api/ranking/request/";

pub struct PlaylistMaker {
    pub data: Arc<Mutex<AppData>>,
    playlist: Playlist
}

impl Default for Playlist {
    fn default() -> Self {
        Self { 
            name: Default::default(), 
            author: Default::default(), 
            description: Default::default(), 
            image: Default::default(), 
            songs: Default::default() 
        }
    }
}

impl PlaylistMaker {

    pub fn new(data: Arc<Mutex<AppData>>) -> Self {
        Self { data, playlist: Default::default() }
    }

    fn make_json_request_internal<T>(url: &str) -> Result<T, Box<dyn Error>> where T: for<'a> Deserialize<'a> {
        let result = reqwest::blocking::get(url)?.text()?;
        let queue_data = serde_json::from_str::<T>(&result)?;
        Ok(queue_data)
    }
    
    fn make_json_request<T>(url: String) -> Option<T> where T: for<'a> Deserialize<'a> {
        match Self::make_json_request_internal::<T>(&url) {
            Ok(resp) => Some(resp),
            Err(_) => None
        }
    }
    
    fn make_difficulty(diff: &Difficulties) -> Difficulty {
        Difficulty{characteristic: "Standard".to_owned(), name: get_diff(&diff.difficulty).to_owned()}
    }
    
    fn make_song(song_name: String, song_hash: String, level_author: String, difficulties: Vec<Difficulty>) -> Song {
        Song{song_name, song_hash, level_author, difficulties}
    }
    
    fn create_song_data(&self, queue_data: QueueData) -> Song {
        let map_info = queue_data.leaderbaord_info;

        let mut song = Self::make_song(map_info.song_name, map_info.song_hash, map_info.level_author, Vec::new());
    
        let url = format!("{SS_MAP_URL}{}", queue_data.request_id);

        match Self::make_json_request::<QueueData>(url) {
            None => (),
            Some(queue_data) => song.difficulties.append(&mut queue_data.difficulties.iter().map(&Self::make_difficulty).collect())
        }

        let mut app_data = lock!(self.data);
        app_data.progress += app_data.process_amount;
        app_data.ctx.request_repaint();

        return song;
    }
    
    fn make_song_list_async(&self, queue_data: Vec<QueueData>) -> Vec<Song> {
        queue_data.into_par_iter().map(|item| self.create_song_data(item)).collect()
    }
    
    fn make_queue_list_async(&self) -> Vec<QueueData> {
        [format!("{SS_URL}top"), format!("{SS_URL}belowTop")].into_par_iter()
            .map(&Self::make_json_request::<Vec<QueueData>>)
            .flatten().flatten().collect()
    }
    
    pub fn make_playlist(mut self) -> Self {
        lock!(self.data).progress = 1.0;
        lock!(self.data).gen_status = GenStatus::GENERATING;
        let queue_data = self.make_queue_list_async();
        lock!(self.data).process_amount = 99.0 / (queue_data.len() as f32);
        let songs = self.make_song_list_async(queue_data);
        let data = lock!(self.data).clone();
        let image = if data.image_path.is_empty() { encode_base64(include_bytes!("../../queue.png").to_vec()) } else { encode_base64_file(&data.image_path) };
        self.playlist = Playlist{name: data.title, author: data.author, description: data.description, image, songs};
        self
    }

    pub fn serialise(&self) -> String {
        let mut buf = Vec::new();
        let formatter = serde_json::ser::PrettyFormatter::with_indent(b"    ");
        let mut ser = serde_json::Serializer::with_formatter(&mut buf, formatter);
        self.playlist.serialize(&mut ser).unwrap_or_default();
        String::from_utf8(buf).unwrap_or("{}".to_string())
    }
}

const fn get_diff(x: &i32) -> &'static str {
    match x {
        1 => "Easy", 3 => "Normal", 5 => "Hard", 7 => "Expert", 9 => "ExpertPlus", _ => "ExpertPlus"
    }
}

pub fn encode_base64(bytes: Vec<u8>) -> String {
    "data:image/png;base64,".to_owned() + &general_purpose::STANDARD.encode(bytes)
}

pub fn encode_base64_file(path: &str) -> String {
    let output = read_file(path);
    if output.is_err() {return "".to_owned()}
    let extension = path.split(".").collect::<Vec<&str>>().get(1).unwrap().to_owned();
    "data:image/".to_owned() +  &extension + ";base64," + &general_purpose::STANDARD.encode(output.unwrap())
}

fn read_file(path: &str) -> io::Result<Vec<u8>> {
    let file = File::open(path)?;
    let mut buffer = Vec::new();
    BufReader::new(file).read_to_end(&mut buffer)?;
    Ok(buffer)
}