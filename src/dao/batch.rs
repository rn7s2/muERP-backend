use crate::models::{batch, prelude::*};
use sea_orm::{
    ActiveModelTrait, ActiveValue, DatabaseConnection, DbBackend, DbErr, EntityTrait,
    FromQueryResult, InsertResult, QuerySelect, Statement,
};

#[derive(rocket_okapi::JsonSchema, rocket::serde::Serialize, rocket::serde::Deserialize)]
#[serde(crate = "rocket::serde")]
#[derive(Debug, FromQueryResult)]
pub struct BatchAndItem {
    pub id: u32,
    pub date: chrono::NaiveDate,
    pub number: i32,
    pub expiration: chrono::NaiveDate,
    pub vendor: Option<String>,
    pub disabled: u8,
    pub item_id: u32,

    pub name: String,
    pub specification: Option<String>,
    pub unit: Option<String>,
    pub manufacturer: String,
    pub price: f32,
}

pub async fn get_batches_and_items(db: &DatabaseConnection) -> Result<Vec<BatchAndItem>, DbErr> {
    Batch::find()
        .from_raw_sql(Statement::from_string(
            DbBackend::MySql,
            r#"SELECT `batch`.*, `item`.`name`,`item`.`specification`,`item`.`unit`,`item`.`manufacturer`,`item`.`price` FROM `item` INNER JOIN `batch` ON `batch`.`item_id`=`item`.`id` ORDER BY `batch`.`date` DESC"#
                .to_string(),
        )).into_model::<BatchAndItem>()
        .all(db)
        .await
}

pub async fn get_max_id(db: &DatabaseConnection) -> Result<u32, DbErr> {
    Ok(Batch::find()
        .column(batch::Column::Id)
        .all(db)
        .await?
        .into_iter()
        .fold(0, |max, x| x.id.max(max)) as u32)
}

pub async fn create_batch(
    db: &DatabaseConnection,
    batch: batch::Model,
) -> Result<InsertResult<batch::ActiveModel>, DbErr> {
    let next_id = get_max_id(db as &DatabaseConnection).await? + 1;

    Batch::insert(batch::ActiveModel {
        id: ActiveValue::Set(batch.id.max(next_id)),
        date: ActiveValue::Set(batch.date),
        number: ActiveValue::Set(batch.number),
        expiration: ActiveValue::Set(batch.expiration),
        vendor: ActiveValue::Set(batch.vendor),
        disabled: ActiveValue::Set(batch.disabled),
        item_id: ActiveValue::Set(batch.item_id),
    })
    .exec(db)
    .await
}

pub async fn disable_batch(db: &DatabaseConnection, id: u32) -> Result<(), DbErr> {
    let batch = Batch::find_by_id(id).one(db).await?;

    let batch = match batch {
        Some(batch) => batch,
        None => return Err(DbErr::RecordNotFound(String::from("Batch not found!"))),
    };

    let active_model = batch::ActiveModel {
        id: ActiveValue::Unchanged(batch.id),
        date: ActiveValue::Unchanged(batch.date),
        number: ActiveValue::Unchanged(batch.number),
        expiration: ActiveValue::Unchanged(batch.expiration),
        vendor: ActiveValue::Unchanged(batch.vendor),
        disabled: ActiveValue::Set(1),
        item_id: ActiveValue::Unchanged(batch.item_id),
    };

    match active_model.update(db).await {
        Ok(_) => Ok(()),
        Err(err) => Err(err),
    }
}
