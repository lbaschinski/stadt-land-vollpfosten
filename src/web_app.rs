use axum::{
    extract::State,
    extract::Form,
    http::{StatusCode, Uri, header},
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use minijinja::{context, Environment};
use rust_embed_for_web::{EmbedableFile, RustEmbed};
use serde::Deserialize;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

use crate::cards;
use crate::dice;

#[derive(RustEmbed)]
#[folder = "src/assets/"]
struct Asset;

/// Game state that is fixed per session
struct GameState {
    categories: Mutex<Vec<String>>,
    round_state: Mutex<RoundState>,
    environment: Environment<'static>,
}

/// RoundState that is cleaned before each new round
struct RoundState {
    timeout: Option<u32>,
    letter: Option<char>,
    reduced_card: Option<Vec<String>>,
    complete_card: Option<Vec<String>>,
    category: Option<String>,
    current_index: Option<usize>,
}

impl RoundState {
    /// Create an empty RoundState
    pub fn empty() -> RoundState {
        Self::new(None, None, None, None, None, None)
    }
    /// Create a new RoundState with the given fields
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
    /// Update a RoundState with the given value
    pub fn update_timeout(&mut self, timeout: Option<u32>) {
        self.timeout = timeout;
    }
    pub fn update_letter(&mut self, letter: Option<char>) {
        self.letter = letter;
    }
    pub fn update_reduced_card(&mut self, reduced_card: Option<Vec<String>>) {
        self.reduced_card = reduced_card;
    }
    pub fn update_complete_card(&mut self, complete_card: Option<Vec<String>>) {
        self.complete_card = complete_card;
    }
    pub fn update_category(&mut self, category: Option<String>) {
        self.category = category;
    }
    pub fn update_current_index(&mut self, current_index: Option<usize>) {
        self.current_index = current_index;
    }
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
/// Input to set up a game session, used to set `GameState`
struct GameInput {
    collection_name: String,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
/// New input for each round, used to set `RoundState`
struct RoundInput {
    timeout: Option<u32>,
    success: Option<bool>,
}

/// Serves the game app and spawns a timeout-checking thread
pub async fn serve() {
    let mut env = Environment::new();
    env.add_template("layout", include_str!("./templates/layout.jinja")).unwrap();
    env.add_template("home", include_str!("./templates/home.jinja")).unwrap();
    env.add_template("categories", include_str!("./templates/categories.jinja")).unwrap();
    env.add_template("start", include_str!("./templates/start.jinja")).unwrap();
    env.add_template("round", include_str!("./templates/round.jinja")).unwrap();
    env.add_template("timer", include_str!("./templates/timer.jinja")).unwrap();
    env.add_template("result", include_str!("./templates/result.jinja")).unwrap();

    // Prepare `GameState` with empty state and the environment
    let game_state = Arc::new(GameState
        { categories: Mutex::new(Vec::new())
        , round_state: Mutex::new(RoundState::empty())
        , environment: env
        });

    // Spawns a timeout thread that counts down the timeout of each round if set
    let handle = spawn_timeout_thread(Arc::clone(&game_state));

    let app = Router::new()
        .route("/", get(handler_home))
        .route("/slv.png", get(get_background_png))
        .route("/start", get(handler_start_game).post(post_start_game))
        .route("/categories", get(handler_categories).post(post_categories))
        .route("/round", get(handler_start_round).post(post_start_round))
        .route("/timer", get(handler_start_timer).post(post_start_timer))
        .route("/result", get(handler_result))
        .route("/*uri", get(not_found))
        .with_state(game_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3333").await.unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
    handle.join().unwrap();
}

/// Spawn a thread that loops indefinitely.
/// Each loop puts the thread to sleep for 1 second and reduces the
/// timeout variable by 1 if set and not 0 already.
fn spawn_timeout_thread(state: Arc<GameState>) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_secs(1));
            let mut round_state = state.round_state.lock().unwrap();
            if round_state.timeout.is_some() {
                let t = round_state.timeout.unwrap();
                round_state.update_timeout(Some(if t > 0 { t - 1 } else { t }));
            };
        };
    })
}

// Fallback route for anything that doesn't match
async fn not_found(State(state): State<Arc<GameState>>, uri: Uri) -> impl IntoResponse {
    // Re-use "home" template since it has the same format
    let template = state.environment.get_template("home").unwrap();

    let path = uri.path().trim_start_matches('/');

    let rendered = template
        .render(context! {
            title => "404",
            welcome_text => format!("Not Found: /{path}"),
        })
        .unwrap();

    (StatusCode::NOT_FOUND, Html(rendered)).into_response()
}

/// Handler for "Home". Does nothing in particular.
async fn handler_home(State(state): State<Arc<GameState>>) -> Result<Html<String>, StatusCode> {
    let template = state.environment.get_template("home").unwrap();

    let rendered = template
        .render(context! {
            title => "Stadt Land Vollpfosten - digital helper",
            welcome_text => "Press 'Start New Game' to start a game!",
        })
        .unwrap();

    Ok(Html(rendered))
}

/// Get handler which responds with the background image.
async fn get_background_png(uri: Uri) -> impl IntoResponse {
    match Asset::get(uri.path().trim_start_matches('/')) {
        Some(content) => {
            ([(header::CONTENT_TYPE, "image/png")], content.data()).into_response()
        }
        None => (StatusCode::NOT_FOUND, "404 Not Found").into_response(),
    }
}

/// Get handler to prepare a game. Simply displays a page to put in a `collection_name`.
/// Clears the `RoundState` too.
async fn handler_start_game(State(state): State<Arc<GameState>>) -> Result<Html<String>, StatusCode> {
    let template = state.environment.get_template("start").unwrap();

    let mut round_state = state.round_state.lock().unwrap();
    *round_state = RoundState::empty();

    let rendered = template
        .render(context! {
            title => "Prepare Game",
        })
        .unwrap();

    Ok(Html(rendered))
}

/// Post handler for preparing a game. Is used when the user wants to delete all saved collections.
/// Work-around - since "input" does not allow for "delete" method and we don't need the post handler
/// for anything else. Clears the `RoundState` too.
async fn post_start_game(State(state): State<Arc<GameState>>) -> Result<Html<String>, StatusCode> {
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

/// Get handler for displaying all categories.
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

/// Post handler for adding new category collections and displaying all afterwards.
async fn post_categories(State(state): State<Arc<GameState>>, Form(input): Form<GameInput>) -> Result<Html<String>, StatusCode> {
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

/// Get handler to start a new round. Displays the "please roll the dice" button.
/// Deletes the `RoundState` again, since the user can reach this without setting up new
/// category collections.
async fn handler_start_round(State(state): State<Arc<GameState>>) -> Result<Html<String>, StatusCode> {
    let template = state.environment.get_template("round").unwrap();

    let mut round_state = state.round_state.lock().unwrap();
    *round_state = RoundState::empty();

    let rendered = template
        .render(context! {
            title => "Start Round",
            first_round => true,
        })
        .unwrap();

    Ok(Html(rendered))
}

/// Post handler for a new round. Handles the dice roll and displays the additional
/// input of the current timeout.
async fn post_start_round(State(state): State<Arc<GameState>>) -> Result<Html<String>, StatusCode> {
    let template = state.environment.get_template("round").unwrap();

    let mut round_state = state.round_state.lock().unwrap();
    if round_state.letter.is_none() {
        let letter = dice::roll_dice().chars().next();
        *round_state = RoundState::empty(); // be sure to empty the state before starting a new round
        round_state.update_letter(letter);
    }

    let rendered = template
        .render(context! {
            title => "Start Round",
            letter => round_state.letter,
        })
        .unwrap();

    Ok(Html(rendered))
}

/// Get handler for a timed round. Is called when a refresh happens, never directly in the app.
/// Simply displays the current round state.
async fn handler_start_timer(State(state): State<Arc<GameState>>) -> Result<Html<String>, StatusCode> {
    let template = state.environment.get_template("timer").unwrap();

    let round_state = state.round_state.lock().unwrap();

    let rendered = template
        .render(context! {
            title => "~ Play ~",
            timeout => round_state.timeout,
            letter => round_state.letter,
            category => round_state.category,
            current_index => round_state.current_index,
            rest => round_state.reduced_card
        })
        .unwrap();

    Ok(Html(rendered))
}

/// Post handler for a timed round. Is called when the user sets the timeout and therefore starts the timed round;
/// drawing the categories that belong to that round; and handling the state when the "Success" or "Next" button are pressed.
async fn post_start_timer(State(state): State<Arc<GameState>>, Form(input): Form<RoundInput>) -> Result<Html<String>, StatusCode> {
    let template = state.environment.get_template("timer").unwrap();
    let category: String;

    let mut round_state = state.round_state.lock().unwrap();
    let categories = state.categories.lock().unwrap();
    let mut current_index = round_state.current_index.unwrap_or(0);
    let mut category_amount;

    let (reduced_card, complete_card) = match &round_state.complete_card {
        // not first round, need to handle the state
        Some(c) => {
            let cc = c.to_vec();
            let mut rc = round_state.reduced_card.clone().unwrap();
            let success = input.success.unwrap();
            category_amount = round_state.reduced_card.as_ref().unwrap().len();
            if success {
                // if successful: remove the category from the set
                let old_category = round_state.category.clone().unwrap();
                let i = rc.iter().position(|x| *x == old_category).unwrap();
                rc.remove(i);
                category_amount -= 1;
                // since an element was removed, do nothing to the index or `-1` (if last element)
                current_index = if i == 0 || i < category_amount { i } else { i - 1 };
            } else {
                // not successful (next button):
                // since no element was removed, either go back to 0 (if last element) or increase index by 1
                current_index = if current_index == (category_amount - 1) { 0 } else { current_index + 1 };
            }
            (rc, cc)
        },
        // first round, need to draw the new categories
        None => {
            category_amount = 6; // potential todo: make this configurable?
            let dc = cards::draw_card(&categories, category_amount as u32);
            (dc.clone(), dc)
        },
    };
    // if all categories were successfully challenged (`reduced_card` is empty), simply put a placeholder
    // since the view does not display a `category` then anyway
    category = if reduced_card.len() == 0 { "".to_string() } else { reduced_card[current_index].clone() };
    // first round only setup
    if input.timeout.is_some() {
        round_state.update_timeout(input.timeout);
        round_state.update_complete_card(Some(complete_card.clone()));
    }
    round_state.update_reduced_card(Some(reduced_card.clone()));
    round_state.update_category(Some(category));
    round_state.update_current_index(Some(current_index));

    let rendered = template
        .render(context! {
            title => "~ Play ~",
            timeout => round_state.timeout,
            letter => round_state.letter,
            category => round_state.category,
            current_index => round_state.current_index,
            rest => round_state.reduced_card
        })
        .unwrap();

    Ok(Html(rendered))
}

/// Get handler to display a rounds results.
/// Better be save than sorry: delete the `RoundState` here too.
async fn handler_result(State(state): State<Arc<GameState>>) -> Result<Html<String>, StatusCode> {
    let template = state.environment.get_template("result").unwrap();

    let mut round_state = state.round_state.lock().unwrap();
    let old_timeout = round_state.timeout.clone();
    let old_letter = round_state.letter.clone();
    let old_card = round_state.complete_card.clone();
    let old_rest = round_state.reduced_card.clone();
    *round_state = RoundState::empty();

    let rendered = template
        .render(context! {
            title => "Round Results",
            timeout => old_timeout,
            letter => old_letter,
            card => old_card,
            rest => old_rest,
        })
        .unwrap();

    Ok(Html(rendered))
}
