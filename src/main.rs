mod utils;
mod handlers;
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
    //load the html pages
    let templates = Tera::new("src/frontend/templates/*.html").unwrap();
    
    //check if the data needed for the game is present

    // aggregate_data() creates the structure needed to query champions
    // data starting from the given jsons
    let champion_list = aggregate_data().await;
    let state = Arc::new(AppState {
        templates,
        champion_list,
    });
    

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


fn _print_type<T>(_: &T) {
    println!("{:?}", std::any::type_name::<T>());
}
