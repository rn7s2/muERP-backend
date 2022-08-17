use crate::dao;
use crate::models::item;
use rocket::{
    delete, get, http, post, put,
    response::status::Custom,
    serde::json::{json, Json, Value},
    State,
};
use rocket_okapi::openapi;
use sea_orm::{DatabaseConnection, DbErr};

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
) -> Result<Json<u32>, Custom<Value>> {
    let result = dao::item::insert_item(db as &DatabaseConnection, item.0).await;

    match result {
        Ok(res) => Ok(Json(res.last_insert_id)),
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

#[openapi(tag = "item")]
#[put("/items/<id>", data = "<item>")]
pub async fn modify_item(
    db: &State<DatabaseConnection>,
    id: u32,
    item: Json<item::Model>,
) -> Result<(), Custom<Value>> {
    if id != item.id {
        return Err(Custom(
            http::Status::Conflict,
            json!({
                "error": {
                    "code": 409,
                    "reason": "Conflict",
                    "description": "Conflict parameters: id does not equal to item.id."
                }
            }),
        ));
    }

    match dao::item::modify_item(db as &DatabaseConnection, item.0).await {
        Ok(_) => Ok(()),
        Err(err) => match err {
            DbErr::RecordNotFound(_) => Err(Custom(
                http::Status::NotFound,
                json!({
                  "error": {
                    "code": 404,
                    "reason": "Item Not Found",
                    "description": "Item not found in the database."
                  }
                }),
            )),
            _ => Err(Custom(
                http::Status::InternalServerError,
                json!({
                  "error": {
                    "code": 500,
                    "reason": "Internal Server Error",
                    "description": "Error occurs while modifying item."
                  }
                }),
            )),
        },
    }
}

#[openapi(tag = "item")]
#[delete("/items/<id>")]
pub async fn delete_item(db: &State<DatabaseConnection>, id: u32) -> Result<(), Custom<Value>> {
    let result = dao::item::delete_item(db as &DatabaseConnection, id).await;

    match result {
        Ok(_) => Ok(()),
        Err(err) => match err {
            DbErr::RecordNotFound(_) => Err(Custom(
                http::Status::NotFound,
                json!({
                  "error": {
                    "code": 404,
                    "reason": "Item Not Found",
                    "description": "Item not found in the database."
                  }
                }),
            )),
            _ => Err(Custom(
                http::Status::InternalServerError,
                json!({
                  "error": {
                    "code": 500,
                    "reason": "Internal Server Error",
                    "description": "Error occurs while deleting item."
                  }
                }),
            )),
        },
    }
}
