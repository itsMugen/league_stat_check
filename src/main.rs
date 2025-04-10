mod handlers;
mod utils;
use crate::utils::retrive_data::{get_data_tarball, retrive_last_patch};
use axum::{
    routing::{get, post},
    Router,
};
use handlers::handlers::{check_stat, stat_check};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::sync::Arc;
use tera::Tera;
use tower_http::services::ServeDir;
use utils::data_parsing::aggregate_data;

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

struct AppState {
    templates: Tera,
    champion_list: HashMap<String, Stats>,
}

#[tokio::main]
async fn main() {
    //load the html pages
    let templates = Tera::new("src/frontend/templates/*.html").unwrap();

    //check if the data needed for the game is present and
    //if it is up to date
    let current_patch = retrive_last_patch().await;
    if !fs::exists(format!("assets/data_tarball/{}", current_patch)).unwrap() {
        get_data_tarball().await;
    }

    // aggregate_data() creates the structure needed to query champions
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
        .nest_service(
            "/champ_images",
            ServeDir::new("assets/data_tarball/img/champion/centered"),
        )
        .nest_service(
            "/stats",
            ServeDir::new("assets/data_tarball/img/perk-images/StatMods"),
        )
        .nest_service(
            "/items",
            ServeDir::new(format!("assets/data_tarball/{}/img/item", current_patch)),
        )
        .nest_service("/scripts", ServeDir::new("src/frontend/scripts"))
        //eventually all used assets/data_tarball will be moved under assets
        .nest_service("/assets", ServeDir::new("assets/"))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
