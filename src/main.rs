mod utils;
use axum::{
    extract::{Form, State},
    response::{Html, IntoResponse},
    routing::{get, post},
    Router,
};
use rand::prelude::*;
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::sync::Arc;
use tera::{Context, Tera};
use tower_http::services::ServeDir;
use crate::utils::retrive_data::retrive_last_patch;

#[derive(Deserialize, Debug, Clone)]
struct Stats {
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

impl Stats {
    pub fn as_list(&self) -> [f32; 9] {
        [
            self.armor.into(),
            self.attackrange.into(),
            self.attackdamage.into(),
            self.attackspeed,
            self.hp.into(),
            self.hpregen,
            self.movespeed.into(),
            self.mp.into(),
            self.spellblock.into(),
        ]
    }
}

#[derive(Deserialize, Debug, Clone)]
struct CheckStats {
    champ_1: String,
    champ_2: String,
    armor: String,
    attackrange: String,
    attackdamage: String,
    attackspeed: String,
    hp: String,
    hpregen: String,
    movespeed: String,
    resource_bar: String,
    magic_resist: String,
}

impl CheckStats {
    pub fn get_guesses(self) -> [String; 9] {
        [
            self.armor,
            self.attackrange,
            self.attackdamage,
            self.attackspeed,
            self.hp,
            self.hpregen,
            self.movespeed,
            self.resource_bar,
            self.magic_resist,
        ]
    }
}

struct AppState {
    templates: Tera,
    champion_list: HashMap<String, Stats>,
}

#[tokio::main]
async fn main() {
    let templates = Tera::new("src/frontend/templates/*.html").unwrap();
    let champion_list = aggregate_data().await;
    let state = Arc::new(AppState {
        templates,
        champion_list,
    });

    // build our application with a single route
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/stat_check", get(stat_check))
        .route("/check_stats", post(check_stat))
        .nest_service("/styles", ServeDir::new("src/frontend/styles"))
        .nest_service("/champ_images", ServeDir::new("assets/data_tarball/img/champion/centered"))
        .nest_service("/stats", ServeDir::new("assets/data_tarball/img/perk-images/StatMods"))
        .nest_service("/items", ServeDir::new(format!("assets/data_tarball/{}/img/item", retrive_last_patch().await)))
        .nest_service("/scripts", ServeDir::new("src/frontend/scripts"))
        //eventually all used assets/data_tarball will be moved under assets
        .nest_service("/assets", ServeDir::new("assets/"))
        .with_state(state);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn check_stat(
    State(state): State<Arc<AppState>>,
    Form(payload): Form<CheckStats>,
) -> impl IntoResponse {
    let champ_list = state.champion_list.clone();
    let stat_1 = champ_list.get(&payload.champ_1).unwrap().as_list();
    let mut stat_2 = champ_list.get(&payload.champ_2).unwrap().as_list();

    //try to swap for an array here
    let mut checked: Vec<String> = Vec::new();
    for (a, b) in stat_1.iter().zip(stat_2.iter_mut()) {
        if a > b {
            checked.push(payload.champ_1.to_string())
        } else if a < b {
            checked.push(payload.champ_2.to_string())
        } else {
            checked.push("draw".to_string())
        }
    }

    let mut score : Vec<bool> = Vec::new();
    let mut score_count: i8 = 0;
    for (guess, truth) in payload.clone().get_guesses().iter().zip(checked.iter_mut()){
        if guess == truth {
            score.push(true);
            score_count+=1;
        } else {
            score.push(false);
        }
    }

    let comment : &str;

    match score_count {
        0 => comment = "are you even trying?",
        1..=3 =>  comment = "I've seen better iron players",
        4..=6 => comment = "Truly a coinflipper",
        7..=8 => comment = "You might actually know something",
        9 => comment = "Meh.. just lucky admit it",
        _ => comment = "You broke something",
    };

    let mut context = Context::new();
    context.insert("comment", &comment);
    context.insert("score", &score_count);
    context.insert("guess_bool", &score);
    context.insert("correct_guess", &checked);
    context.insert("player_guess", &payload.get_guesses());
    let rendered = state.templates.render("score.html", &context).unwrap();

    Html(rendered)
}

async fn stat_check(State(state): State<Arc<AppState>>) -> Html<String> {
    //pick 2 random champs
    let champ_map = state.champion_list.clone();
    let mut rng = rand::rng();
    let mut champ_names: Vec<String> = champ_map.into_keys().collect();
    champ_names.shuffle(&mut rng);
    let mut champs = champ_names.choose_multiple(&mut rng, 2);
    let champ_1 = champs.next();
    let champ_2 = champs.next();
    

    //create page
    let mut context = Context::new();
    if champ_1.unwrap() == "Fiddlesticks" {
        context.insert("champ_1", "FiddleSticks");
        context.insert("champ_2", &champ_2.unwrap());
    } else if champ_2.unwrap() == "Fiddlesticks" {
        context.insert("champ_1", &champ_1.unwrap());
        context.insert("champ_2", "FiddleSticks");
    } else {
        context.insert("champ_1", &champ_1.unwrap());
        context.insert("champ_2", &champ_2.unwrap());
    }

    let rendered = state.templates.render("base.html", &context).unwrap();
    Html(rendered)
}

async fn aggregate_data() -> HashMap<String, Stats> {
    println!("{:#?}", format!("assets/data_tarball/{}/data/en_US/champion", retrive_last_patch().await));
    let champs_dir = fs::read_dir(format!("assets/data_tarball/{}/data/en_US/champion", retrive_last_patch().await)).unwrap();
    let mut champs: HashMap<String, Stats> = HashMap::new();
    for champ_path in champs_dir {
        let path = champ_path.unwrap();
        let champ_file = &path.file_name().into_string().unwrap();
        let champ_name = champ_file.split(".").collect::<Vec<_>>()[0];

        let champ_json = fs::read(&path.path());

        let champ_data: Value = serde_json::from_slice(&champ_json.unwrap()).expect("some data");

        let stats: Stats =
            serde_json::from_value(champ_data["data"][champ_name]["stats"].clone()).unwrap();

        champs.insert(champ_name.to_string(), stats);
    }
    champs
}

fn _print_type<T>(_: &T) {
    println!("{:?}", std::any::type_name::<T>());
}
