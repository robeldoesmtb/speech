// src/levels/loader.rs
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::BufReader;

#[derive(Deserialize, Serialize, Debug)]
pub struct LevelData {
    pub name: String,
    pub perspective: Perspective,
    pub platforms: Vec<Platform>,
    pub evidence: Vec<Evidence>,
    pub spawn_point: (f32, f32),
    pub exit_point: (f32, f32),
}

#[derive(Deserialize, Serialize, Debug)]
pub enum Perspective {
    SideScrolling,
    TopDown,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Platform {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Evidence {
    pub x: f32,
    pub y: f32,
    pub id: String,
    pub points: u32,
}

pub fn load_level(level_id: usize) -> Result<LevelData, Box<dyn std::error::Error>> {
    let file = File::open(format!("assets/levels/level_{}.json", level_id))?;
    let reader = BufReader::new(file);
    let level_data = serde_json::from_reader(reader)?;
    Ok(level_data)
}