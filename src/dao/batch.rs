use crate::models::{batch, item, prelude::*};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, ConnectionTrait, DatabaseConnection,
    DatabaseTransaction, DbBackend, DbErr, EntityTrait, FromQueryResult, QueryFilter, QuerySelect,
    Statement, TransactionTrait,
};

#[derive(rocket_okapi::JsonSchema, rocket::serde::Serialize, rocket::serde::Deserialize)]
#[serde(crate = "rocket::serde")]
#[derive(Debug, FromQueryResult)]
pub struct StockInAndItem {
    pub number: i32,
    pub item_id: u32,
    pub name: String,
    pub specification: Option<String>,
    pub unit: Option<String>,
    pub manufacturer: String,
    pub price: f32,
}

pub async fn get_stock_in_and_items(
    db: &DatabaseConnection,
    from_date: String,
    to_date: String,
) -> Result<Vec<StockInAndItem>, DbErr> {
    Item::find()
        .from_raw_sql(Statement::from_string(
            DbBackend::MySql,
            format!("SELECT CAST(SUM(`batch`.`number`) as INTEGER) AS `number`, `batch`.`item_id`, `item`.`name`,`item`.`specification`,`item`.`unit`,`item`.`manufacturer`,`item`.`price` FROM `item` INNER JOIN `batch` ON `batch`.`item_id`=`item`.`id` WHERE `batch`.`date`>= \"{}\" AND `batch`.`date`<= \"{}\" GROUP BY `batch`.`item_id` ORDER BY `item`.`price` DESC", from_date, to_date),
        )).into_model::<StockInAndItem>()
        .all(db)
        .await
}

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
            r#"SELECT `batch`.*, `item`.`name`,`item`.`specification`,`item`.`unit`,`item`.`manufacturer`,`item`.`price` FROM `item` INNER JOIN `batch` ON `batch`.`item_id`=`item`.`id` ORDER BY `batch`.`date` DESC, `batch`.`id` DESC"#
                .to_string(),
        )).into_model::<BatchAndItem>()
        .all(db)
        .await
}

pub async fn get_max_id<T: ConnectionTrait>(db: &T) -> Result<u32, DbErr> {
    Ok(Batch::find()
        .column(batch::Column::Id)
        .all(db)
        .await?
        .into_iter()
        .fold(0, |max, x| x.id.max(max)) as u32)
}

pub async fn create_batch_transaction(
    transaction: &DatabaseTransaction,
    batch: batch::Model,
) -> Result<(), DbErr> {
    let next_id = get_max_id(transaction).await? + 1;

    Batch::insert(batch::ActiveModel {
        id: ActiveValue::Set(batch.id.max(next_id)),
        date: ActiveValue::Set(batch.date),
        number: ActiveValue::Set(batch.number),
        expiration: ActiveValue::Set(batch.expiration),
        vendor: ActiveValue::Set(batch.vendor),
        disabled: ActiveValue::Set(batch.disabled),
        item_id: ActiveValue::Set(batch.item_id),
    })
    .exec(transaction)
    .await?;

    let item = match Item::find_by_id(batch.item_id).one(transaction).await? {
        Some(item) => item,
        None => return Err(DbErr::RecordNotFound(String::from("Item not found."))),
    };

    let active_model = item::ActiveModel {
        id: ActiveValue::Unchanged(item.id),
        name: ActiveValue::Unchanged(item.name),
        specification: ActiveValue::Unchanged(item.specification),
        unit: ActiveValue::Unchanged(item.unit),
        manufacturer: ActiveValue::Unchanged(item.manufacturer),
        number: ActiveValue::Set(item.number + batch.number),
        price: ActiveValue::Unchanged(item.price),
        expiration: ActiveValue::Set(item.expiration.min(batch.expiration)),
    };
    active_model.update(transaction).await?;

    Ok(())
}

pub async fn create_batch(db: &DatabaseConnection, batch: batch::Model) -> Result<(), DbErr> {
    let transaction = db.begin().await?;

    let next_id = get_max_id(&transaction).await? + 1;

    Batch::insert(batch::ActiveModel {
        id: ActiveValue::Set(batch.id.max(next_id)),
        date: ActiveValue::Set(batch.date),
        number: ActiveValue::Set(batch.number),
        expiration: ActiveValue::Set(batch.expiration),
        vendor: ActiveValue::Set(batch.vendor),
        disabled: ActiveValue::Set(batch.disabled),
        item_id: ActiveValue::Set(batch.item_id),
    })
    .exec(&transaction)
    .await?;

    let item = match Item::find_by_id(batch.item_id).one(&transaction).await? {
        Some(item) => item,
        None => return Err(DbErr::RecordNotFound(String::from("Item not found."))),
    };

    let active_model = item::ActiveModel {
        id: ActiveValue::Unchanged(item.id),
        name: ActiveValue::Unchanged(item.name),
        specification: ActiveValue::Unchanged(item.specification),
        unit: ActiveValue::Unchanged(item.unit),
        manufacturer: ActiveValue::Unchanged(item.manufacturer),
        number: ActiveValue::Set(item.number + batch.number),
        price: ActiveValue::Unchanged(item.price),
        expiration: ActiveValue::Set(item.expiration.min(batch.expiration)),
    };
    active_model.update(&transaction).await?;

    transaction.commit().await
}

pub async fn disable_batch(db: &DatabaseConnection, id: u32) -> Result<(), DbErr> {
    let transaction = db.begin().await?;

    let batch = Batch::find_by_id(id).one(&transaction).await?;
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
    active_model.update(&transaction).await?;

    let min_expiration = Batch::find()
        .filter(batch::Column::ItemId.eq(batch.item_id))
        .filter(batch::Column::Disabled.ne(1))
        .all(&transaction)
        .await?
        .into_iter()
        .fold(chrono::NaiveDate::from_ymd(2099, 12, 31), |min, item| {
            if item.expiration < min {
                item.expiration
            } else {
                min
            }
        });

    let item = match Item::find_by_id(batch.item_id).one(&transaction).await? {
        Some(item) => item,
        None => return Err(DbErr::RecordNotFound(String::from("Item not found."))),
    };
    let active_model = item::ActiveModel {
        id: ActiveValue::Unchanged(item.id),
        name: ActiveValue::Unchanged(item.name),
        specification: ActiveValue::Unchanged(item.specification),
        unit: ActiveValue::Unchanged(item.unit),
        manufacturer: ActiveValue::Unchanged(item.manufacturer),
        number: ActiveValue::Unchanged(item.number),
        price: ActiveValue::Unchanged(item.price),
        expiration: ActiveValue::Set(min_expiration),
    };
    active_model.update(&transaction).await?;

    transaction.commit().await
}
