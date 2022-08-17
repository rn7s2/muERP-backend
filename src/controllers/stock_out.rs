use crate::dao;
use crate::models::stock_out;
use rocket::{
    get, http, post,
    response::status::Custom,
    serde::json::{json, Json, Value},
    State,
};
use rocket_okapi::openapi;
use sea_orm::DatabaseConnection;

#[openapi(tag = "stock-out")]
#[get("/stock-out-and-items")]
pub async fn get_stock_out_and_items(
    db: &State<DatabaseConnection>,
) -> Result<Json<Vec<dao::stock_out::StockOutAndItem>>, Custom<Value>> {
    let stock_outs = dao::stock_out::get_stock_out_and_items(db as &DatabaseConnection).await;

    match stock_outs {
        Ok(stock_outs) => Ok(Json(stock_outs)),
        Err(_) => Err(Custom(
            http::Status::InternalServerError,
            json!({
              "error": {
                "code": 500,
                "reason": "Internal Server Error",
                "description": "Error occurs while getting stock_out records from the database."
              }
            }),
        )),
    }
}

#[openapi(tag = "stock-out")]
#[post("/stock-out", data = "<stock_out>")]
pub async fn insert_or_update_stock_out(
    db: &State<DatabaseConnection>,
    stock_out: Json<stock_out::Model>,
) -> Result<(), Custom<Value>> {
    let result = dao::stock_out::insert_or_update_stock_out(db, stock_out.0).await;

    match result {
        Ok(_) => Ok(()),
        Err(_) => Err(Custom(
            http::Status::InternalServerError,
            json!({
              "error": {
                "code": 500,
                "reason": "Internal Server Error",
                "description": "Error occurs while inserting or updating StockOut record."
              }
            }),
        )),
    }
}

#[openapi(tag = "stock-out")]
#[get("/stock-out/<id>")]
pub async fn get_stock_out_by_item_id(
    db: &State<DatabaseConnection>,
    id: u32,
) -> Result<Json<Vec<stock_out::Model>>, Custom<Value>> {
    let stock_out = dao::stock_out::get_stock_out_by_item_id(db as &DatabaseConnection, id).await;

    match stock_out {
        Ok(stock_out) => Ok(Json(stock_out)),
        Err(_) => Err(Custom(
            http::Status::NotFound,
            json!({
                "error": {
                "code": 500,
                "reason": "Internal Server Error",
                "description": "Error occurs while getting stock_outs records from the database."
            }}),
        )),
    }
}
