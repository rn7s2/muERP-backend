use crate::models::batch;
use crate::{dao, models::item};
extern crate umya_spreadsheet;
use chrono;
use rocket::{
    form::Form,
    fs::TempFile,
    get, http, patch, post,
    response::status::Custom,
    serde::json::{json, Json, Value},
    FromForm, State,
};
use rocket_okapi::openapi;
use sea_orm::{DatabaseConnection, TransactionTrait};
use std::{env::temp_dir, ops::Add};
use uuid::Uuid;

#[openapi(tag = "batch")]
#[get("/stock-in-and-items?<from>&<to>")]
pub async fn get_stock_in_and_items(
    db: &State<DatabaseConnection>,
    from: String,
    to: String,
) -> Result<Json<Vec<dao::batch::StockInAndItem>>, Custom<Value>> {
    let result = dao::batch::get_stock_in_and_items(db, from, to).await;

    match result {
        Ok(stock_in_and_items) => Ok(Json(stock_in_and_items)),
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

#[derive(FromForm)]
pub struct Upload<'r> {
    file: TempFile<'r>,
}

#[post("/batches-from-xlsx", data = "<upload>")]
pub async fn create_batch_from_xlsx(
    db: &State<DatabaseConnection>,
    mut upload: Form<Upload<'_>>,
) -> Result<(), Custom<Value>> {
    let mut tmp_dir = temp_dir();
    tmp_dir.push(format!("{}.xlsx", Uuid::new_v4()));

    let file = match tmp_dir.to_str() {
        Some(file) => file.to_string(),
        None => {
            return Err(Custom(
                http::Status::InternalServerError,
                json!({
                  "error": {
                    "code": 500,
                    "reason": "Internal Server Error",
                    "description": "Error occurs while saving the xlsx file."
                  }
                }),
            ))
        }
    };
    match upload.file.persist_to(&file).await {
        Ok(_) => (),
        Err(_) => {
            return Err(Custom(
                http::Status::InternalServerError,
                json!({
                  "error": {
                    "code": 500,
                    "reason": "Internal Server Error",
                    "description": "Error occurs while saving the xlsx file."
                  }
                }),
            ))
        }
    };

    let path = std::path::Path::new(&file);
    let book = match umya_spreadsheet::reader::xlsx::read(path) {
        Ok(result) => result,
        Err(_) => {
            match rocket::tokio::fs::remove_file(path).await {
                Ok(_) => (),
                Err(_) => {
                    return Err(Custom(
                        http::Status::InternalServerError,
                        json!({
                          "error": {
                            "code": 500,
                            "reason": "Internal Server Error",
                            "description": "Error occurs while deleting the xlsx file."
                          }
                        }),
                    ))
                }
            };
            return Err(Custom(
                http::Status::InternalServerError,
                json!({
                  "error": {
                    "code": 500,
                    "reason": "Internal Server Error",
                    "description": "Error occurs while selecting workbook from the xlsx file."
                  }
                }),
            ));
        }
    };

    let delete_and_error = || -> Result<(), Custom<Value>> {
        match std::fs::remove_file(path) {
            Ok(_) => (),
            Err(_) => {
                return Err(Custom(
                    http::Status::InternalServerError,
                    json!({
                      "error": {
                        "code": 500,
                        "reason": "Internal Server Error",
                        "description": "Error occurs while deleting the xlsx file."
                      }
                    }),
                ))
            }
        };
        return Err(Custom(
            http::Status::InternalServerError,
            json!({
              "error": {
                "code": 500,
                "reason": "Internal Server Error",
                "description": "Error occurs while selecting sheet from the xlsx file."
              }
            }),
        ));
    };

    let sheet = match book.get_sheet(&0) {
        Ok(result) => result,
        Err(_) => return delete_and_error(),
    };

    let transaction = match db.begin().await {
        Ok(transaction) => transaction,
        Err(_) => return delete_and_error(),
    };
    let items = match dao::item::get_items(&transaction).await {
        Ok(items) => items,
        Err(_) => return delete_and_error(),
    };

    let get_cell_value_string = |col, row| {
        sheet
            .get_cell_value_by_column_and_row(&col, &row)
            .get_value()
            .to_string()
    };

    for i in 2..=sheet.get_highest_row() {
        let date = chrono::NaiveDate::from_ymd(1900, 1, 1).add(chrono::Duration::days(
            match get_cell_value_string(1, i).parse::<i64>() {
                Ok(val) => val,
                Err(_) => return delete_and_error(),
            } - 2,
        ));
        let name = get_cell_value_string(2, i);
        let specification = get_cell_value_string(3, i);
        let unit = get_cell_value_string(4, i);
        let manufacturer = get_cell_value_string(5, i);
        let number = match get_cell_value_string(6, i).parse::<i32>() {
            Ok(number) => number,
            Err(_) => return delete_and_error(),
        };
        let price = match get_cell_value_string(7, i).parse::<f32>() {
            Ok(val) => val,
            Err(_) => return delete_and_error(),
        };
        let expiration = chrono::NaiveDate::from_ymd(1900, 1, 1).add(chrono::Duration::days(
            match get_cell_value_string(8, i).parse::<i64>() {
                Ok(val) => val,
                Err(_) => return delete_and_error(),
            } - 2,
        ));
        let vendor = get_cell_value_string(9, i);

        let item_matched: Vec<&item::Model> = items
            .iter()
            .filter(|item| item.name == name && item.manufacturer == manufacturer)
            .collect();

        if item_matched.len() == 1 {
            match dao::batch::create_batch_transaction(
                &transaction,
                batch::Model {
                    id: 0,
                    date,
                    number,
                    expiration,
                    vendor: Some(vendor),
                    disabled: 0,
                    item_id: item_matched[0].id,
                },
            )
            .await
            {
                Ok(_) => (),
                Err(_) => return delete_and_error(),
            };
        } else {
            let item_id = match dao::item::insert_item_transaction(
                &transaction,
                item::Model {
                    id: 0,
                    name,
                    specification: Some(specification),
                    unit: Some(unit),
                    manufacturer,
                    number: 0,
                    price,
                    expiration: chrono::NaiveDate::from_ymd(2099, 12, 31),
                },
            )
            .await
            {
                Ok(val) => val,
                Err(_) => return delete_and_error(),
            }
            .last_insert_id;

            match dao::batch::create_batch_transaction(
                &transaction,
                batch::Model {
                    id: 0,
                    date,
                    number,
                    expiration,
                    vendor: Some(vendor),
                    disabled: 0,
                    item_id: item_id,
                },
            )
            .await
            {
                Ok(_) => (),
                Err(_) => return delete_and_error(),
            };
        }
    }

    match transaction.commit().await {
        Ok(_) => (),
        Err(_) => return delete_and_error(),
    }

    match rocket::tokio::fs::remove_file(path).await {
        Ok(_) => Ok(()),
        Err(_) => {
            return Err(Custom(
                http::Status::InternalServerError,
                json!({
                  "error": {
                    "code": 500,
                    "reason": "Internal Server Error",
                    "description": "Error occurs while deleting the xlsx file."
                  }
                }),
            ))
        }
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
