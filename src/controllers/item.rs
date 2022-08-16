use super::super::entities::{item, prelude::Item};
use rocket::{get, serde::json::Json, State};
use rocket_okapi::openapi;
use sea_orm::{DatabaseConnection, EntityTrait};

#[openapi(tag = "stocker-vue")]
#[get("/items")]
pub async fn index(db: &State<DatabaseConnection>) -> Json<Vec<item::Model>> {
    let db = db as &DatabaseConnection;

    let names = Item::find().into_model().all(db).await.unwrap();

    Json(names)
}
