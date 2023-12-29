use axum::extract::{Query, State};
use axum::{http::StatusCode, response::IntoResponse, routing::get, Json, Router};
use rust_indexed::indexers::{SearchIndex, SearchResult};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use tokio::task;

struct AppState {
    page_index: RwLock<SearchIndex>,
}

#[tokio::main]
async fn main() {
    // initialize tracing
    // tracing_subscriber::fmt::init();

    let app_state = Arc::new(AppState {
        page_index: RwLock::new(SearchIndex::open("index_page").unwrap()),
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
    dbg!(&params);
    let q = params.q;
    let _ = params.page.unwrap_or(1);

    let results = task::spawn_blocking(move || state.page_index.read().unwrap().search(&q))
        .await
        .unwrap()
        .unwrap();

    dbg!(&results);

    (StatusCode::OK, Json(SearchResponse { results }))
}

// the output to our `search` handler
#[derive(Serialize)]
struct SearchResponse {
    results: Vec<SearchResult>,
}

#[derive(Debug, Deserialize)]
struct Params {
    q: String,
    page: Option<u32>,
}
