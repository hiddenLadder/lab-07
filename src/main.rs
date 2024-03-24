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
) -> Result<Json<PriceDto>, StatusCode> {
    let id = Uuid::new_v4();
    prices.write().unwrap().push(PriceDto {
        id,
        price: input.price,
    });
    Ok(Json(PriceDto {
        id,
        price: input.price,
    }))
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
) -> Result<Json<PriceDto>, StatusCode> {
    let mut prices = prices.write().unwrap();
    if let Some(price) = prices.iter_mut().find(|p| p.id == id) {
        price.price = input.price;
        return Ok(Json(*price));
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

#[cfg(test)]
mod tests {
    use super::*;
    use ::axum_test::TestServer;
    use serde_json::json;

    #[tokio::test]
    async fn test_get_prices_empty() {
        let state = Arc::new(RwLock::new(vec![]));
        let app = app(state.clone());
        let server = TestServer::new(app).unwrap();

        let response = server.get("/prices").await;
        assert_eq!(response.status_code(), StatusCode::OK);
        assert_eq!(response.text(), json!([]).to_string());
    }

    #[tokio::test]
    async fn test_get_prices_populated() {
        let id = Uuid::new_v4();
        let prices = vec![PriceDto { id, price: 100 }];
        let state: AppState = Arc::new(RwLock::new(prices.clone()));
        let app = app(state);
        let server = TestServer::new(app).unwrap();

        let response = server.get("/prices").await;

        assert_eq!(response.status_code(), 200);
        assert_eq!(response.text(), json!(prices).to_string());
    }

    #[tokio::test]
    async fn test_create_price() {
        let state: AppState = Arc::new(RwLock::new(vec![]));
        let app = app(state.clone());
        let server = TestServer::new(app).unwrap();

        let response = server
            .post("/prices")
            .json(&CreatePrice { price: 100 })
            .await;

        let price_dto = response.json::<PriceDto>();
        assert_eq!(response.status_code(), StatusCode::OK);
        assert!(state.read().unwrap().iter().any(|p| p.id == price_dto.id));
        assert_eq!(state.read().unwrap().len(), 1);
        assert_eq!(state.read().unwrap()[0].price, 100);
    }

    #[tokio::test]
    async fn test_get_price_existent() {
        let id = Uuid::new_v4();
        let state = Arc::new(RwLock::new(vec![PriceDto { id, price: 3000 }]));
        let app = app(state.clone());
        let server = TestServer::new(app).unwrap();

        let response = server.get(&format!("/prices/{}", id)).await;
        assert_eq!(response.status_code(), StatusCode::OK);
        assert_eq!(
            response.text(),
            json!({"id": id, "price": 3000}).to_string()
        );
    }

    #[tokio::test]
    async fn test_get_price_nonexistent() {
        let id = Uuid::new_v4();
        let state = Arc::new(RwLock::new(vec![]));
        let app = app(state.clone());
        let server = TestServer::new(app).unwrap();

        let response = server.get(&format!("/prices/{}", id)).await;
        assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_update_price_existent() {
        let id = Uuid::new_v4();
        let state = Arc::new(RwLock::new(vec![PriceDto { id, price: 4000 }]));
        let app = app(state.clone());
        let server = TestServer::new(app).unwrap();

        let response = server
            .patch(&format!("/prices/{}", id))
            .json(&CreatePrice { price: 5000 })
            .await;

        assert_eq!(response.status_code(), StatusCode::OK);
        assert_eq!(
            response.text(),
            json!({"id": id, "price": 5000}).to_string()
        );
    }

    #[tokio::test]
    async fn test_update_price_nonexistent() {
        let id = Uuid::new_v4();
        let state = Arc::new(RwLock::new(vec![]));
        let app = app(state.clone());
        let server = TestServer::new(app).unwrap();

        let response = server
            .patch(&format!("/prices/{}", id))
            .json(&CreatePrice { price: 6000 })
            .await;
        assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_delete_price_existent() {
        let id = Uuid::new_v4();
        let state = Arc::new(RwLock::new(vec![PriceDto { id, price: 7000 }]));
        let app = app(state.clone());
        let server = TestServer::new(app).unwrap();

        let response = server.delete(&format!("/prices/{}", id)).await;
        assert_eq!(response.status_code(), StatusCode::OK);

        assert_eq!(state.read().unwrap().len(), 0);
    }

    #[tokio::test]
    async fn test_delete_price_nonexistent() {
        let id = Uuid::new_v4();
        let state = Arc::new(RwLock::new(vec![]));
        let app = app(state.clone());
        let server = TestServer::new(app).unwrap();

        let response = server.delete(&format!("/prices/{}", id)).await;
        assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
    }
}
