use crate::models::{item, prelude::*, stock_out};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, ConnectionTrait, DatabaseConnection, DbBackend,
    DbErr, EntityTrait, FromQueryResult, QueryFilter, QueryOrder, QuerySelect, Statement,
    TransactionTrait,
};

#[derive(rocket_okapi::JsonSchema, rocket::serde::Serialize, rocket::serde::Deserialize)]
#[serde(crate = "rocket::serde")]
#[derive(Debug, FromQueryResult)]
pub struct StockOutAndItem {
    pub number: i32,
    pub item_id: u32,
    pub name: String,
    pub specification: Option<String>,
    pub unit: Option<String>,
    pub manufacturer: String,
    pub price: f32,
}

pub async fn get_stock_out_and_items(
    db: &DatabaseConnection,
    from_date: String,
    to_date: String,
) -> Result<Vec<StockOutAndItem>, DbErr> {
    Item::find()
        .from_raw_sql(Statement::from_string(
            DbBackend::MySql,
            format!("SELECT CAST(SUM(`stock_out`.`number`) as INTEGER) AS `number`, `stock_out`.`item_id`, `item`.`name`,`item`.`specification`,`item`.`unit`,`item`.`manufacturer`,`item`.`price` FROM `item` INNER JOIN `stock_out` ON `stock_out`.`item_id`=`item`.`id` WHERE `stock_out`.`date`>= \"{}\" AND `stock_out`.`date`<= \"{}\" GROUP BY `stock_out`.`item_id` ORDER BY `item`.`price` DESC", from_date, to_date),
        )).into_model::<StockOutAndItem>()
        .all(db)
        .await
}

pub async fn get_stock_out_by_item_id<T: ConnectionTrait>(
    db: &T,
    id: u32,
) -> Result<Vec<stock_out::Model>, DbErr> {
    StockOut::find()
        .filter(stock_out::Column::ItemId.eq(id))
        .order_by_desc(stock_out::Column::Date)
        .all(db)
        .await
}

pub async fn get_max_id<T: ConnectionTrait>(db: &T) -> Result<u32, DbErr> {
    Ok(StockOut::find()
        .column(stock_out::Column::Id)
        .all(db)
        .await?
        .into_iter()
        .fold(0, |max, x| x.id.max(max)) as u32)
}

pub async fn insert_or_update_stock_out(
    db: &DatabaseConnection,
    stock_out: stock_out::Model,
) -> Result<(), DbErr> {
    let transaction = db.begin().await?;

    let mut stock_outs = get_stock_out_by_item_id(&transaction, stock_out.item_id).await?;
    stock_outs.sort_by(|a, b| a.date.cmp(&b.date));

    let exists = stock_outs.iter().fold(
        false,
        |ans, s| if s.date == stock_out.date { true } else { ans },
    );
    let next_id = get_max_id(&transaction).await? + 1;

    if stock_outs.len() == 0 || !exists {
        StockOut::insert(stock_out::ActiveModel {
            id: ActiveValue::Set(stock_out.id.max(next_id)),
            date: ActiveValue::Set(stock_out.date),
            number: ActiveValue::Set(stock_out.number),
            item_id: ActiveValue::Set(stock_out.item_id),
        })
        .exec(&transaction)
        .await?;
    } else {
        let last_record = stock_outs
            .iter()
            .filter(|s| s.date == stock_out.date)
            .last()
            .unwrap();

        let active_model = stock_out::ActiveModel {
            id: ActiveValue::Unchanged(last_record.id),
            date: ActiveValue::Unchanged(last_record.date),
            number: ActiveValue::Set(last_record.number + stock_out.number),
            item_id: ActiveValue::Unchanged(last_record.item_id),
        };
        active_model.update(&transaction).await?;
    }

    let item = match Item::find_by_id(stock_out.item_id)
        .one(&transaction)
        .await?
    {
        Some(item) => item,
        None => return Err(DbErr::RecordNotFound(String::from("Item not found."))),
    };

    let active_model = item::ActiveModel {
        id: ActiveValue::Unchanged(item.id),
        name: ActiveValue::Unchanged(item.name),
        specification: ActiveValue::Unchanged(item.specification),
        unit: ActiveValue::Unchanged(item.unit),
        manufacturer: ActiveValue::Unchanged(item.manufacturer),
        number: ActiveValue::Set(item.number - stock_out.number),
        price: ActiveValue::Unchanged(item.price),
        expiration: ActiveValue::Unchanged(item.expiration),
    };
    active_model.update(&transaction).await?;

    transaction.commit().await
}
