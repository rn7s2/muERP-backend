use crate::dao;
use crate::models::item;
use rocket::{
    get, http, post,
    response::status::Custom,
    serde::json::{json, Json, Value},
    State,
};
use rocket_okapi::openapi;
use sea_orm::DatabaseConnection;

#[openapi(tag = "item")]
#[get("/items")]
pub async fn get_items(
    db: &State<DatabaseConnection>,
) -> Result<Json<Vec<item::Model>>, Custom<Value>> {
    let items = dao::item::get_items(db as &DatabaseConnection).await;

    match items {
        Ok(items) => Ok(Json(items)),
        Err(_) => Err(Custom(
            http::Status::InternalServerError,
            json!({
              "error": {
                "code": 500,
                "reason": "Internal Server Error",
                "description": "Error occurs while getting items from the database."
              }
            }),
        )),
    }
}

#[openapi(tag = "item")]
#[post("/items", data = "<item>")]
pub async fn create_item(
    db: &State<DatabaseConnection>,
    item: Json<item::Model>,
) -> Result<(), Custom<Value>> {
    let result = dao::item::insert_item(db as &DatabaseConnection, item.0).await;

    match result {
        Ok(_) => Ok(()),
        Err(_) => Err(Custom(
            http::Status::InternalServerError,
            json!({
              "error": {
                "code": 500,
                "reason": "Internal Server Error",
                "description": "Error occurs while inserting new item into the database."
              }
            }),
        )),
    }
}
