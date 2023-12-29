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
        .route("/search/", get(search))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    // tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn search(
    Query(params): Query<Params>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
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

    println!("{} results. duration = {:?}", results.len(), duration);

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
