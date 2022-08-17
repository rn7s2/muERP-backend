use crate::models::{item, prelude::*, stock_out};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, ConnectionTrait, DatabaseConnection, DbErr,
    EntityTrait, QueryFilter, QueryOrder, QuerySelect, TransactionTrait,
};

pub async fn get_stock_out(db: &DatabaseConnection) -> Result<Vec<stock_out::Model>, DbErr> {
    StockOut::find().into_model().all(db).await
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
