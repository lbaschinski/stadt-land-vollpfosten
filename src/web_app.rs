use axum::{
    extract::State,
    extract::Form,
    http::StatusCode,
    response::Html,
    routing::get,
    Router,
};
use minijinja::{context, Environment};
use serde::Deserialize;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

use crate::cards;
use crate::dice;

struct GameState {
    categories: Mutex<Vec<String>>,
    round_state: Mutex<RoundState>,
    environment: Environment<'static>,
}

struct RoundState {
    timeout: Option<u32>,
    letter: Option<char>,
    reduced_card: Option<Vec<String>>,
    complete_card: Option<Vec<String>>,
    category: Option<String>,
    current_index: Option<usize>,
}

impl RoundState {
    pub fn empty() -> RoundState {
        Self::new(None, None, None, None, None, None)
    }
    pub fn new
        ( timeout: Option<u32>
        , letter: Option<char>
        , reduced_card: Option<Vec<String>>
        , complete_card: Option<Vec<String>>
        , category: Option<String>
        , current_index: Option<usize>
    ) -> RoundState {
        RoundState { timeout, letter, reduced_card, complete_card, category, current_index }
    }
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct GameInput {
    collection_name: String,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct RoundInput {
    timeout: Option<u32>,
    success: Option<bool>,
}

pub async fn serve() {
    let mut env = Environment::new();
    env.add_template("layout", include_str!("./templates/layout.jinja")).unwrap();
    env.add_template("home", include_str!("./templates/home.jinja")).unwrap();
    env.add_template("categories", include_str!("./templates/categories.jinja")).unwrap();
    env.add_template("start", include_str!("./templates/start.jinja")).unwrap();
    env.add_template("round", include_str!("./templates/round.jinja")).unwrap();
    env.add_template("timer", include_str!("./templates/timer.jinja")).unwrap();
    env.add_template("result", include_str!("./templates/result.jinja")).unwrap();

    let game_state = Arc::new(GameState
        { categories: Mutex::new(Vec::new())
        , round_state: Mutex::new(RoundState::empty())
        , environment: env
        });

    let handle = spawn_timeout_thread(Arc::clone(&game_state));

    let app = Router::new()
        .route("/", get(handler_home))
        .route("/start", get(handler_start_game).post(post_start_game))
        .route("/categories", get(handler_categories))
        .route("/round", get(handler_start_round).post(post_start_round))
        .route("/timer", get(handler_start_timer).post(post_start_timer))
        .route("/result", get(handler_result))
        .with_state(game_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3333").await.unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
    handle.join().unwrap();
}

fn spawn_timeout_thread(state: Arc<GameState>) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_secs(1));
            let mut round_state = state.round_state.lock().unwrap();
            *round_state = match round_state.timeout {
                Some(t) => RoundState::new
                    ( Some(if t > 0 { t - 1 } else { t })
                    , round_state.letter
                    , round_state.reduced_card.clone()
                    , round_state.complete_card.clone()
                    , round_state.category.clone()
                    , round_state.current_index
                    ),
                None => RoundState::new
                    ( round_state.timeout
                    , round_state.letter
                    , round_state.reduced_card.clone()
                    , round_state.complete_card.clone()
                    , round_state.category.clone()
                    , round_state.current_index
                    ),
            };
        };
    })
}

async fn handler_home(State(state): State<Arc<GameState>>) -> Result<Html<String>, StatusCode> {
    let template = state.environment.get_template("home").unwrap();

    let rendered = template
        .render(context! {
            title => "Stadt Land Vollpfosten - digital helper",
            welcome_text => "Press 'Start Game' to start a game!",
        })
        .unwrap();

    Ok(Html(rendered))
}

async fn handler_start_game(State(state): State<Arc<GameState>>) -> Result<Html<String>, StatusCode> {
    let template = state.environment.get_template("start").unwrap();

    let mut categories = state.categories.lock().unwrap();
    *categories = Vec::new();
    let mut round_state = state.round_state.lock().unwrap();
    *round_state = RoundState::empty();

    let rendered = template
        .render(context! {
            title => "Prepare Game",
        })
        .unwrap();

    Ok(Html(rendered))
}

async fn post_start_game(State(state): State<Arc<GameState>>, Form(input): Form<GameInput>) -> Result<Html<String>, StatusCode> {
    let template = state.environment.get_template("categories").unwrap();
    let categories = cards::load_categories(&input.collection_name);
    {
        let mut cat = state.categories.lock().unwrap();
        cat.extend(categories.clone());
    }

    let rendered = template
        .render(context! {
            title => "Categories",
            categories => state.categories,
        })
        .unwrap();

    Ok(Html(rendered))
}

async fn handler_categories(State(state): State<Arc<GameState>>) -> Result<Html<String>, StatusCode> {
    let template = state.environment.get_template("categories").unwrap();

    let rendered = template
        .render(context! {
            title => "Categories",
            categories => state.categories,
        })
        .unwrap();

    Ok(Html(rendered))
}

async fn handler_start_round(State(state): State<Arc<GameState>>) -> Result<Html<String>, StatusCode> {
    let template = state.environment.get_template("round").unwrap();

    let rendered = template
        .render(context! {
            title => "Start Round",
            first_round => true,
        })
        .unwrap();

    Ok(Html(rendered))
}

async fn post_start_round(State(state): State<Arc<GameState>>) -> Result<Html<String>, StatusCode> {
    let template = state.environment.get_template("round").unwrap();

    let mut round_state = state.round_state.lock().unwrap();
    if round_state.letter.is_none() {
        let letter = dice::roll_dice().chars().next();
        *round_state = RoundState::new(None, letter, None, None, None, None);
    }

    let rendered = template
        .render(context! {
            title => "Start Round",
            letter => round_state.letter,
        })
        .unwrap();

    Ok(Html(rendered))
}

// Needed for refreshing the page to show the running timer (never called directly)
async fn handler_start_timer(State(state): State<Arc<GameState>>) -> Result<Html<String>, StatusCode> {
    let template = state.environment.get_template("timer").unwrap();

    let round_state = state.round_state.lock().unwrap();

    let rendered = template
        .render(context! {
            title => "Start Timer",
            timeout => round_state.timeout,
            letter => round_state.letter,
            category => round_state.category,
            current_index => round_state.current_index,
            rest => round_state.reduced_card
        })
        .unwrap();

    Ok(Html(rendered))
}

async fn post_start_timer(State(state): State<Arc<GameState>>, Form(input): Form<RoundInput>) -> Result<Html<String>, StatusCode> {
    let template = state.environment.get_template("timer").unwrap();
    let category: String;

    let mut round_state = state.round_state.lock().unwrap();
    let categories = state.categories.lock().unwrap();
    let mut current_index = round_state.current_index.unwrap_or(0);
    let mut category_amount;

    let (reduced_card, complete_card) = match &round_state.complete_card {
        Some(c) => {
            let cc = c.to_vec();
            let mut rc = round_state.reduced_card.clone().unwrap();
            let success = input.success.unwrap();
            category_amount = round_state.reduced_card.as_ref().unwrap().len();
            if success {
                let old_category = round_state.category.clone().unwrap();
                let i = rc.iter().position(|x| *x == old_category).unwrap();
                rc.remove(i);
                category_amount -= 1;
                // since an element was removed, do nothing to the index or `-1` (if last element)
                current_index = if i == 0 || i < category_amount { i } else { i - 1 };
            } else {
                // since no element was removed, either go back to 0 (if last element) or `+1`
                current_index = if current_index == (category_amount - 1) { 0 } else { current_index + 1 };
            }
            (rc, cc)
        },
        None => {
            category_amount = 6;
            let dc = cards::draw_card(&categories, category_amount as u32);
            (dc.clone(), dc)
        },
    };
    category = reduced_card[current_index].clone();
    *round_state = RoundState::new
        ( if input.timeout.is_some() { input.timeout } else { round_state.timeout }
        , round_state.letter
        , Some(reduced_card.clone())
        , Some(complete_card.clone())
        , Some(category)
        , Some(current_index)
    );

    let rendered = template
        .render(context! {
            title => "Start Timer",
            timeout => round_state.timeout,
            letter => round_state.letter,
            category => round_state.category,
            current_index => round_state.current_index,
            rest => round_state.reduced_card
        })
        .unwrap();

    Ok(Html(rendered))
}

async fn handler_result(State(state): State<Arc<GameState>>) -> Result<Html<String>, StatusCode> {
    let template = state.environment.get_template("result").unwrap();

    let mut round_state = state.round_state.lock().unwrap();
    let old_timeout = round_state.timeout.clone();
    let old_letter = round_state.letter.clone();
    let old_card = round_state.complete_card.clone();
    *round_state = RoundState::empty();

    let rendered = template
        .render(context! {
            title => "Round Results",
            timeout => old_timeout,
            letter => old_letter,
            card => old_card,
        })
        .unwrap();

    Ok(Html(rendered))
}
