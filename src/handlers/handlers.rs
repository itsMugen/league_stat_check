use crate::AppState;
use axum::extract::State;
use axum::response::Html;
use axum::response::IntoResponse;
use axum::Form;
use rand::prelude::*;
use serde::Deserialize;
use std::sync::Arc;
use tera::Context;

#[derive(Deserialize, Debug, Clone)]
pub struct CheckStats {
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

pub async fn stat_check(State(state): State<Arc<AppState>>) -> Html<String> {
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
    context.insert("champ_1", &champ_1.unwrap());
    context.insert("champ_2", &champ_2.unwrap());

    let rendered = state.templates.render("base.html", &context).unwrap();
    Html(rendered)
}

pub async fn check_stat(
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

    let mut score: Vec<bool> = Vec::new();
    let mut score_count: i8 = 0;
    for (guess, truth) in payload.clone().get_guesses().iter().zip(checked.iter_mut()) {
        if guess == truth {
            score.push(true);
            score_count += 1;
        } else {
            score.push(false);
        }
    }

    let comment: &str;

    match score_count {
        0 => comment = "are you even trying?",
        1..=3 => comment = "I've seen better iron players",
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
