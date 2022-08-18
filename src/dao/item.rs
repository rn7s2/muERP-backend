use crate::models::{batch, item, prelude::*, stock_out};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, ConnectionTrait, DatabaseConnection,
    DatabaseTransaction, DbErr, EntityTrait, InsertResult, QueryFilter, QuerySelect,
    TransactionTrait,
};

pub async fn get_items<T: ConnectionTrait>(db: &T) -> Result<Vec<item::Model>, DbErr> {
    Item::find().into_model().all(db).await
}

pub async fn get_max_id<T: ConnectionTrait>(db: &T) -> Result<u32, DbErr> {
    Ok(Item::find()
        .column(item::Column::Id)
        .all(db)
        .await?
        .into_iter()
        .fold(0, |max, x| x.id.max(max)) as u32)
}

pub async fn insert_item_transaction(
    transaction: &DatabaseTransaction,
    item: item::Model,
) -> Result<InsertResult<item::ActiveModel>, DbErr> {
    let next_id = get_max_id(transaction).await? + 1;

    Item::insert(item::ActiveModel {
        id: ActiveValue::Set(item.id.max(next_id)),
        name: ActiveValue::Set(item.name.clone()),
        specification: ActiveValue::Set(item.specification.clone()),
        unit: ActiveValue::Set(item.unit.clone()),
        manufacturer: ActiveValue::Set(item.manufacturer.clone()),
        number: ActiveValue::Set(item.number),
        price: ActiveValue::Set(item.price),
        expiration: ActiveValue::Set(item.expiration),
    })
    .exec(transaction)
    .await
}

pub async fn insert_item(
    db: &DatabaseConnection,
    item: item::Model,
) -> Result<InsertResult<item::ActiveModel>, DbErr> {
    let next_id = get_max_id(db as &DatabaseConnection).await? + 1;

    Item::insert(item::ActiveModel {
        id: ActiveValue::Set(item.id.max(next_id)),
        name: ActiveValue::Set(item.name.clone()),
        specification: ActiveValue::Set(item.specification.clone()),
        unit: ActiveValue::Set(item.unit.clone()),
        manufacturer: ActiveValue::Set(item.manufacturer.clone()),
        number: ActiveValue::Set(item.number),
        price: ActiveValue::Set(item.price),
        expiration: ActiveValue::Set(item.expiration),
    })
    .exec(db)
    .await
}

pub async fn modify_item(db: &DatabaseConnection, item: item::Model) -> Result<item::Model, DbErr> {
    let item = item::ActiveModel {
        id: ActiveValue::Set(item.id),
        name: ActiveValue::Set(item.name.clone()),
        specification: ActiveValue::Set(item.specification.clone()),
        unit: ActiveValue::Set(item.unit.clone()),
        manufacturer: ActiveValue::Set(item.manufacturer.clone()),
        number: ActiveValue::Set(item.number),
        price: ActiveValue::Set(item.price),
        expiration: ActiveValue::Set(item.expiration),
    };

    item.update(db).await
}

pub async fn delete_item(db: &DatabaseConnection, id: u32) -> Result<(), DbErr> {
    let transaction = db.begin().await?;

    Batch::delete_many()
        .filter(batch::Column::ItemId.eq(id))
        .exec(&transaction)
        .await?;
    StockOut::delete_many()
        .filter(stock_out::Column::ItemId.eq(id))
        .exec(&transaction)
        .await?;
    Item::delete_by_id(id).exec(&transaction).await?;

    transaction.commit().await
}
