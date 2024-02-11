use axum::extract::{Query, State};
use axum::{http::StatusCode, response::IntoResponse, routing::get, Json, Router};
use rust_indexed::index::SearchResult;
use rust_indexed::ranking::Ranking;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use std::thread::sleep;
use std::time::Duration;
use std::time::Instant;
use tokio::task;
use tower_http::services::ServeDir;

struct AppState {
    page_index: RwLock<Ranking>,
}

#[tokio::main]
async fn main() {
    // initialize tracing
    // tracing_subscriber::fmt::init();

    let app_state = Arc::new(AppState {
        page_index: RwLock::new(Ranking::default()),
    });

    let app = Router::new()
        .route("/search/", get(search)) // API
        .nest_service("/", ServeDir::new("app")) // Static (for local development)
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();
}

async fn search(
    Query(params): Query<Params>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let q_debug = params.q.clone();
    let q = params.q;
    let _ = params.page.unwrap_or(1);

    let start = Instant::now();

    let results = task::spawn_blocking(move || {
        sleep(Duration::from_millis(200));
        state.page_index.read().unwrap().search(&q)
    })
    .await
    .unwrap();

    let duration = start.elapsed();

    println!(
        "{} results. duration = {:?} query = `{}`",
        results.len(),
        duration,
        q_debug
    );

    (
        StatusCode::OK,
        Json(SearchResponse {
            results,
            duration_milis: duration.as_millis(),
        }),
    )
}

// the output to our `search` handler
#[derive(Serialize)]
struct SearchResponse {
    results: Vec<SearchResult>,
    duration_milis: u128,
}

#[derive(Debug, Deserialize)]
struct Params {
    q: String,
    page: Option<u32>,
}
