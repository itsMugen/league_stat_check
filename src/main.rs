use axum::{
    extract::State,
    response::Html,
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

struct AppState {
    templates: Tera,
    champion_list: HashMap<String, Stats>,
}

#[tokio::main]
async fn main() {
    let templates = Tera::new("src/frontend/templates/*.html").unwrap();
    let champion_list = aggregate_data();
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
        .nest_service("/champ_images", ServeDir::new("attic/img/champion/splash"))
        .nest_service("/stats", ServeDir::new("attic/img/perk-images/StatMods"))
        .nest_service("/items", ServeDir::new("attic/15.5.1/img/item"))
        .with_state(state);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn check_stat(State(_state): State<Arc<AppState>>) -> Html<String> {
    Html("dunno mate".to_string())
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
        context.insert("champion_name_1", "FiddleSticks");
        context.insert("champion_name_2", &champ_2.unwrap());
    } else if champ_2.unwrap() == "Fiddlesticks" {
        context.insert("champion_name_1", &champ_1.unwrap());
        context.insert("champion_name_2", "FiddleSticks");
    } else {
        context.insert("champion_name_1", &champ_1.unwrap());
        context.insert("champion_name_2", &champ_2.unwrap());
    }

    let rendered = state.templates.render("base.html", &context).unwrap();
    Html(rendered)
}

fn aggregate_data() -> HashMap<String, Stats> {
    let champs_dir = fs::read_dir("attic/15.5.1/data/en_US/champion").unwrap();
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

fn print_type<T>(_: &T) {
    println!("{:?}", std::any::type_name::<T>());
}
