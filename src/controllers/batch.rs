use crate::dao;
use crate::models::batch;
use rocket::{
    get, http, patch, post,
    response::status::Custom,
    serde::json::{json, Json, Value},
    State,
};
use rocket_okapi::openapi;
use sea_orm::DatabaseConnection;

#[openapi(tag = "batch")]
#[get("/batches-and-items")]
pub async fn get_batches_and_items(
    db: &State<DatabaseConnection>,
) -> Result<Json<Vec<dao::batch::BatchAndItem>>, Custom<Value>> {
    let result = dao::batch::get_batches_and_items(db).await;

    match result {
        Ok(batches_and_items) => Ok(Json(batches_and_items)),
        Err(_) => Err(Custom(
            http::Status::InternalServerError,
            json!({
              "error": {
                "code": 500,
                "reason": "Internal Server Error",
                "description": "Error occurs while getting batches from the database."
              }
            }),
        )),
    }
}

#[openapi(tag = "batch")]
#[post("/batches", data = "<batch>")]
pub async fn create_batch(
    db: &State<DatabaseConnection>,
    batch: Json<batch::Model>,
) -> Result<(), Custom<Value>> {
    let result = dao::batch::create_batch(db, batch.0).await;

    match result {
        Ok(_) => Ok(()),
        Err(_) => Err(Custom(
            http::Status::InternalServerError,
            json!({
              "error": {
                "code": 500,
                "reason": "Internal Server Error",
                "description": "Error occurs while getting batches from the database."
              }
            }),
        )),
    }
}

#[openapi(tag = "batch")]
#[patch("/batches/<id>")]
pub async fn disable_batch(db: &State<DatabaseConnection>, id: u32) -> Result<(), Custom<Value>> {
    let result = dao::batch::disable_batch(db, id).await;

    match result {
        Ok(_) => Ok(()),
        Err(_) => Err(Custom(
            http::Status::InternalServerError,
            json!({
              "error": {
                "code": 500,
                "reason": "Internal Server Error",
                "description": "Error occurs while disabling batch from the database."
              }
            }),
        )),
    }
}
