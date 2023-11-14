use std::collections::HashMap;
use serde::Deserialize;
use crate::resources::load_string;


#[derive(Clone, Copy)]
pub struct Tile {
    pub(crate) char:u8,
    pub(crate) color:[f32;3]
}

impl Default for Tile {
    fn default() -> Self {
        Tile{
            char : 0x00,
            color: [1.0,1.0,1.0]
        }
    }
}


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
    world_map:String,

}


impl GameConfig {
    pub async fn new()->Self{
        let str = load_string("game_config.toml").await.unwrap();
        let config : GameConfig = toml::from_str(&str).unwrap();

        // config.state_message = "ROBIN      HP 34/34    Gold:72\nPaladin Lvl 6        XP:18,390";
        config
    }

    pub fn get_map(&self)->Vec<Tile>{
        let mut tile_set = self.map.world.iter().flat_map(|line|{
           line.chars()
               .filter_map(|c|{ self.characters.get(&c) })
               .map(|c| {
                   Tile {
                       char : c.char,
                       color : self.color[c.color]
                   }
               })
               .collect::<Vec<_>>()
        }).collect::<Vec<_>>();


        let status_message = " ROBIN      HP 34/34    Gold:72  Paladin Lvl 6        XP:18,390";
        for char in status_message.chars() {
            tile_set.push(Tile{
                char: char as u8,
                color : [1.0,1.0,1.0]
            })
        }
        return tile_set;
    }
}

pub const SCREEN_ROWS: usize =24;
pub const SCREEN_COLS: usize =32;
pub const CHR_UV:f32 = 0.0625;