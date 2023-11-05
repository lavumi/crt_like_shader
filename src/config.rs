use std::collections::HashMap;
use serde::Deserialize;
use crate::resources::load_string;

#[derive(Debug, Deserialize)]
pub struct Character {
    color: usize,
    char: u8,
    solid: bool,
}

#[derive(Debug, Deserialize)]
pub struct Options {
    pub screen_size: [u32; 2],
}

#[derive(Debug, Deserialize)]
pub struct Map {
    pub world : Vec<String>
}

#[derive(Debug, Deserialize)]
pub struct GameConfig {
    pub characters : HashMap<char, Character>,
    pub options : Options,
    pub color:Vec<[f32;3]>,
    pub map:Map,

    #[serde(default)]
    world_map:String
}


impl GameConfig {
    pub async fn new()->Self{
        let str = load_string("game_config.toml").await.unwrap();
        let config : GameConfig = toml::from_str(&str).unwrap();
        config
    }

    pub fn get_map(&self)->Vec<(u8, [f32;3])>{
        self.map.world.iter().flat_map(|line|{
           line.chars()
               .filter_map(|c|{ self.characters.get(&c) })
               .map(|c| { (
                   c.char, self.color[c.color])
               })
               .collect::<Vec<_>>()
        }).collect::<Vec<_>>()

    }
}

pub const SCREEN_ROWS: u32 =24;
pub const SCREEN_COLS: u32 =32;
pub const CHR_UV:f32 = 0.0625;