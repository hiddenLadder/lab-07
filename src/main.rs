use std::sync::Arc;

use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
struct PriceDto {
    id: Uuid,
    price: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct CreatePrice {
    price: u64,
}

type AppState = Arc<RwLock<Vec<PriceDto>>>;

#[tokio::main]
async fn main() {
    let state = Arc::new(RwLock::new(vec![]));
    let app = app(state);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();
}

fn app(state: AppState) -> Router {
    Router::new()
        .route("/prices", get(get_prices).post(create_price))
        .route(
            "/prices/:id",
            get(get_price).patch(update_price).delete(delete_price),
        )
        .with_state(state)
}

async fn get_prices(State(prices): State<AppState>) -> Json<Vec<PriceDto>> {
    Json(prices.read().unwrap().to_vec())
}

async fn create_price(
    State(prices): State<AppState>,
    Json(input): Json<CreatePrice>,
) -> Json<PriceDto> {
    let id = Uuid::new_v4();
    prices.write().unwrap().push(PriceDto {
        id,
        price: input.price,
    });
    Json(PriceDto {
        id,
        price: input.price,
    })
}

async fn get_price(
    State(prices): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<PriceDto>, StatusCode> {
    let prices = prices.read().unwrap().to_vec();
    if let Some(price) = prices.iter().find(|p| p.id == id) {
        return Ok(Json(price.to_owned()));
    }
    Err(StatusCode::NOT_FOUND)
}

async fn update_price(
    State(prices): State<AppState>,
    Path(id): Path<Uuid>,
    Json(input): Json<CreatePrice>,
) -> Result<StatusCode, StatusCode> {
    let mut prices = prices.write().unwrap();
    if let Some(price) = prices.iter_mut().find(|p| p.id == id) {
        price.price = input.price;
        return Ok(StatusCode::OK);
    }
    Err(StatusCode::NOT_FOUND)
}

async fn delete_price(
    State(prices): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let mut prices = prices.write().unwrap();
    if let Some(price) = prices.iter().position(|p| p.id == id) {
        prices.remove(price);
        return Ok(StatusCode::OK);
    }
    Err(StatusCode::NOT_FOUND)
}
