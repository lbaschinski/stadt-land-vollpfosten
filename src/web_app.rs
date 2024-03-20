use axum::{
    extract::State,
    extract::Form,
    http::StatusCode,
    response::Html,
    routing::{get, post},
    Router,
};
use minijinja::{context, Environment};
use serde::Deserialize;
use std::sync::Arc;
use std::sync::Mutex;
use std::ops::Deref;
use crate::cards;
use crate::dice;

struct AppState {
    categories: Mutex<Vec<String>>,
    timeout: Mutex<u32>,
    letter: Mutex<Option<char>>,
    card: Mutex<Option<Vec<String>>>,
    environment: Environment<'static>,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct PrepareInput {
    category: String,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct GameInput {
    timeout: u32,
    letter: Option<char>,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct RoundInput {
    timeout: u32,
    letter: char,
    current: Option<usize>,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct ResultInput {
    timeout: u32,
    letter: char,
}

pub async fn serve() {
    let mut env = Environment::new();
    env.add_template("layout", include_str!("./templates/layout.jinja")).unwrap();
    env.add_template("home", include_str!("./templates/home.jinja")).unwrap();
    env.add_template("categories", include_str!("./templates/categories.jinja")).unwrap();
    env.add_template("prepare", include_str!("./templates/prepare.jinja")).unwrap();
    env.add_template("start", include_str!("./templates/start.jinja")).unwrap();
    env.add_template("round", include_str!("./templates/round.jinja")).unwrap();
    env.add_template("result", include_str!("./templates/result.jinja")).unwrap();

    let app_state = Arc::new(AppState
        { categories: Mutex::new(Vec::new())
        , timeout: Mutex::new(0)
        , letter: Mutex::new(None)
        , card: Mutex::new(None)
        , environment: env
        });

    let app = Router::new()
        .route("/", get(handler_home))
        .route("/prepare", get(handler_prepare_game).post(post_prepare_game))
        .route("/categories", get(handler_categories))
        .route("/start", get(handler_start_game).post(post_start_game))
        .route("/round", post(post_start_round))
        .route("/result", post(post_result))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3333").await.unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn handler_home(State(state): State<Arc<AppState>>) -> Result<Html<String>, StatusCode> {
    let template = state.environment.get_template("home").unwrap();

    let rendered = template
        .render(context! {
            title => "Stadt Land Vollpfosten - digital helper",
            welcome_text => "Press 'Start Game' to start a game!",
        })
        .unwrap();

    Ok(Html(rendered))
}

async fn handler_prepare_game(State(state): State<Arc<AppState>>) -> Result<Html<String>, StatusCode> {
    let template = state.environment.get_template("prepare").unwrap();

    let rendered = template
        .render(context! {
            title => "Prepare Game",
        })
        .unwrap();

    Ok(Html(rendered))
}

async fn post_prepare_game(State(state): State<Arc<AppState>>, Form(input): Form<PrepareInput>) -> Result<Html<String>, StatusCode> {
    let template = state.environment.get_template("categories").unwrap();
    let categories = cards::load_categories(&input.category);
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

async fn handler_categories(State(state): State<Arc<AppState>>) -> Result<Html<String>, StatusCode> {
    let template = state.environment.get_template("categories").unwrap();

    let rendered = template
        .render(context! {
            title => "Categories",
            categories => state.categories,
        })
        .unwrap();

    Ok(Html(rendered))
}

async fn handler_start_game(State(state): State<Arc<AppState>>) -> Result<Html<String>, StatusCode> {
    let template = state.environment.get_template("start").unwrap();

    let rendered = template
        .render(context! {
            title => "Start Game",
            timeout => state.timeout,
            letter => state.letter,
        })
        .unwrap();

    Ok(Html(rendered))
}

async fn post_start_game(State(state): State<Arc<AppState>>, Form(input): Form<GameInput>) -> Result<Html<String>, StatusCode> {
    let template = state.environment.get_template("start").unwrap();
    {
        let mut timeout = state.timeout.lock().unwrap();
        *timeout = input.timeout;
        let mut letter = state.letter.lock().unwrap();
        *letter = input.letter;
    }

    if input.letter.is_some() {
        let mut letter = state.letter.lock().unwrap();
        *letter = dice::roll_dice().chars().next();
    }

    let rendered = template
        .render(context! {
            title => "Start Game",
            timeout => state.timeout,
            letter => state.letter,
        })
        .unwrap();

    Ok(Html(rendered))
}

async fn post_start_round(State(state): State<Arc<AppState>>, Form(input): Form<RoundInput>) -> Result<Html<String>, StatusCode> {
    let template = state.environment.get_template("round").unwrap();
    let category: String;
    let next: usize;
    let drawn_card: Vec<String>;
    {
        let mut timeout = state.timeout.lock().unwrap();
        *timeout = input.timeout;
        let mut letter = state.letter.lock().unwrap();
        *letter = Some(input.letter);
        // TODO: handle empty list when prepare was not done
        let categories = state.categories.lock().unwrap();
        let mut card = state.card.lock().unwrap();
        drawn_card = match card.deref() {
            Some(c) => c.to_vec(),
            None => cards::draw_card(&categories, 6),
        };
        *card = Some(drawn_card.clone());
        let current_num = match input.current {
            Some(num) => num,
            None => 0,
        };
        next = if current_num == 5 { 0 } else { current_num + 1 };
        category = drawn_card[current_num].clone();
    }

    let rendered = template
        .render(context! {
            title => "Start Round",
            timeout => state.timeout,
            letter => state.letter,
            category => category,
            next => next,
            card => state.card,
        })
        .unwrap();

    Ok(Html(rendered))
}

async fn post_result(State(state): State<Arc<AppState>>, Form(input): Form<ResultInput>) -> Result<Html<String>, StatusCode> {
    let template = state.environment.get_template("result").unwrap();

    let rendered = template
        .render(context! {
            title => "Round Results",
            timeout => input.timeout,
            letter => input.letter,
            card => state.card,
        })
        .unwrap();

    Ok(Html(rendered))
}
