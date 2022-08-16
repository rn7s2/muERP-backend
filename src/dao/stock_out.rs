use crate::models::{prelude::*, stock_out};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, DbErr, EntityTrait,
    QueryFilter, QuerySelect,
};

pub async fn get_stock_out(db: &DatabaseConnection) -> Result<Vec<stock_out::Model>, DbErr> {
    StockOut::find().into_model().all(db).await
}

pub async fn get_stock_out_by_item_id(
    db: &DatabaseConnection,
    id: u32,
) -> Result<Vec<stock_out::Model>, DbErr> {
    StockOut::find()
        .filter(stock_out::Column::ItemId.eq(id))
        .all(db)
        .await
}

pub async fn get_max_id(db: &DatabaseConnection) -> Result<u32, DbErr> {
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
    let mut stock_outs = get_stock_out_by_item_id(db, stock_out.item_id).await?;
    stock_outs.sort_by(|a, b| a.date.cmp(&b.date));

    let exists = stock_outs.iter().fold(
        false,
        |ans, s| if s.date == stock_out.date { true } else { ans },
    );

    if stock_outs.len() == 0 || !exists {
        let next_id = get_max_id(db).await? + 1;

        match StockOut::insert(stock_out::ActiveModel {
            id: ActiveValue::Set(stock_out.id.max(next_id)),
            date: ActiveValue::Set(stock_out.date),
            number: ActiveValue::Set(stock_out.number),
            item_id: ActiveValue::Set(stock_out.item_id),
        })
        .exec(db)
        .await
        {
            Ok(_) => Ok(()),
            Err(err) => Err(err),
        }
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

        match active_model.update(db).await {
            Ok(_) => Ok(()),
            Err(err) => Err(err),
        }
    }
}
