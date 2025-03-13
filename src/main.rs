use axum::{
    extract::State,
    response::Html,
    routing::{get, post},
    Router,
};
use rand::prelude::*;
use serde::Deserialize;
use serde_json::Value;
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

#[derive(Deserialize, Debug, Clone)]
struct Champion {
    name: String,
    stats: Stats,
}

impl Champion {
    fn get(&self, field: &str) -> f32{
        match field {
            "armor" => f32::from(self.stats.armor),
            "attackrange" => f32::from(self.stats.attackrange),
            "attackdamage" => f32::from(self.stats.attackdamage),
            "attackspeed" => f32::from(self.stats.attackspeed),
            "hp" => f32::from(self.stats.hp),
            "hpregen" => f32::from(self.stats.hpregen),
            "movespeed" => f32::from(self.stats.movespeed),
            "mp" => f32::from(self.stats.mp),
            "spellblock" => f32::from(self.stats.spellblock),
            _ => 0.0,
        }
    }
}

struct AppState {
    templates: Tera,
    champion_list: Vec<Champion>,
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
        .nest_service(
            "/champ_images",
            ServeDir::new("attic/img/champion/centered"),
        )
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
    let mut champs_list = state.champion_list.clone();
    let mut rng = rand::rng();
    champs_list.shuffle(&mut rng);
    let mut champs = champs_list.choose_multiple(&mut rng, 2);
    let champ_1 = champs.next();
    let champ_2 = champs.next();

    //pick a stat
    let mut stat_rng = rand::rng();
    let random_stat = match stat_rng.gen_range(0..8) {
        0 => "armor",
        1 => "attackrange",
        2 => "attackdamage",
        3 => "attackspeed",
        4 => "hp",
        5 => "hpregen",
        6 => "movespeed",
        7 => "mp",
        8 => "spellblock",
        _ => "nada",
    };

    //create page
    let mut context = Context::new();
    context.insert("champion_name_1", &champ_1.unwrap().name);
    context.insert("champion_name_2", &champ_2.unwrap().name);
    context.insert("stat", &random_stat);

    println!("{:#?}", &champ_2.unwrap().get(&random_stat));

    let rendered = state.templates.render("base.html", &context).unwrap();
    Html(rendered)
}

fn aggregate_data() -> Vec<Champion> {
    let champs_dir = fs::read_dir("attic/15.5.1/data/en_US/champion").unwrap();
    let mut champs: Vec<Champion> = Vec::new();
    for champ_path in champs_dir {
        let path = champ_path.unwrap();
        let champ_file = &path.file_name().into_string().unwrap();
        let champ_name = champ_file.split(".").collect::<Vec<_>>()[0];

        let champ_json = fs::read(&path.path());

        let champ_data: Value = serde_json::from_slice(&champ_json.unwrap()).expect("some data");

        let stats: Stats =
            serde_json::from_value(champ_data["data"][champ_name]["stats"].clone()).unwrap();

        let champion: Champion = Champion {
            name: champ_name.to_string(),
            stats: stats,
        };

        champs.push(champion);
    }
    champs
}

fn print_type<T>(_: &T) {
    println!("{:?}", std::any::type_name::<T>());
}
