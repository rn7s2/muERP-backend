use crate::models::{item, prelude::Item};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseConnection, DbErr, EntityTrait, QuerySelect};

pub async fn get_items(db: &DatabaseConnection) -> Result<Vec<item::Model>, DbErr> {
    Item::find().into_model().all(db).await
}

pub async fn get_max_id(db: &DatabaseConnection) -> Result<u32, DbErr> {
    match Item::find().column(item::Column::Id).all(db).await {
        Ok(ids) => Ok(ids.into_iter().fold(0, |max, x| x.id.max(max)) as u32),
        Err(err) => Err(err),
    }
}

pub async fn insert_item(db: &DatabaseConnection, item: item::Model) -> Result<(), DbErr> {
    let next_id = match get_max_id(db as &DatabaseConnection).await {
        Ok(max_id) => max_id + 1,
        Err(err) => return Err(err),
    };

    match Item::insert(item::ActiveModel {
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
    {
        Ok(_) => Ok(()),
        Err(err) => Err(err),
    }
}

pub async fn modify_item(db: &DatabaseConnection, item: item::Model) -> Result<(), DbErr> {
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

    match item.update(db).await {
        Ok(_) => Ok(()),
        Err(err) => Err(err),
    }
}

pub async fn delete_item(db: &DatabaseConnection, id: u32) -> Result<(), DbErr> {
    todo!("{}", "Deleting needs batches and stock_out to be ready.")
}
