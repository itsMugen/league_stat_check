use std::fs;
use serde_json::Value;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct Stats{
    armor: u16,
    attackrange: u16,
    attackdamage: u16,
    attackspeed: f32,
    hp: u16,
    hpregen: f32,
    movespeed: u16,
    mp: u16,
    spellblock: u16,
}

#[derive(Deserialize, Debug)]
struct Champion{
    name: String,
    stats: Stats,
}

fn main() {
    aggregate_data();
}

fn aggregate_data(){
    let champs_dir= fs::read_dir("attic/15.5.1/data/en_US/champion").unwrap();
    let mut champs: Vec<Champion> = Vec::new(); 
    for champ_path in champs_dir{
        let path = champ_path.unwrap();
        let champ_file = &path.file_name().into_string().unwrap();
        let champ_name = champ_file.split(".").collect::<Vec<_>>()[0];

        let champ_json = fs::read(&path.path());

        let champ_data : Value = serde_json::from_slice(&champ_json.unwrap()).expect("some data");

        let stats : Stats = serde_json::from_value(champ_data["data"][champ_name]["stats"].clone()).unwrap(); 
        
        let champion : Champion = Champion{
            name: champ_name.to_string(),
            stats: stats,
        };

        println!("{:#?}", &champion);
        champs.push(champion);
    }

    //println!("{:#?}", champs[0]["data"]["Graves"]["stats"]);
}

fn print_type<T>(_: &T) { 
    println!("{:?}", std::any::type_name::<T>());
}
