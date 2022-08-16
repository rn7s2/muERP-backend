mod entities;

use entities::{item, prelude::*};
use rocket::{
    catch, catchers, get,
    serde::json::{json, Json, Value},
    State,
};
use rocket_okapi::{
    openapi, openapi_get_routes,
    swagger_ui::{make_swagger_ui, SwaggerUIConfig},
};
use sea_orm::{
    ConnectionTrait, Database, DatabaseConnection, DbBackend, DbErr, EntityTrait, Statement,
};

#[openapi(tag = "stocker-vue")]
#[get("/items")]
async fn index(db: &State<DatabaseConnection>) -> Json<Vec<item::Model>> {
    let db = db as &DatabaseConnection;

    let names = Item::find().into_model().all(db).await.unwrap();

    Json(names)
}

#[catch(404)]
fn not_found() -> Value {
    json!({
        "status": "error",
        "reason": "Resource was not found."
    })
}

async fn setup_db() -> Result<DatabaseConnection, DbErr> {
    // Environment variable DATABASE_URL
    // mysql://<user>:<password>@<host>:<port>
    let database_url = match std::env::var("DATABASE_URL") {
        Ok(url) => url,
        Err(_) => {
            return Err(DbErr::Custom(
                "Environment variable 'DATABASE_URL' not found.".to_string(),
            ))
        }
    };

    let db = Database::connect(&database_url).await?;

    let db_name = "stocker-vue";
    match db.get_database_backend() {
        DbBackend::MySql => {
            db.execute(Statement::from_string(
                db.get_database_backend(),
                format!("CREATE DATABASE IF NOT EXISTS `{}`;", db_name),
            ))
            .await?;

            let url = format!("{}/{}", &database_url, db_name);
            Ok(Database::connect(&url).await?)
        }
        _ => Err(DbErr::Custom("Unsupported database detected.".to_string())),
    }
}

#[rocket::main]
async fn main() {
    let db = match setup_db().await {
        Ok(db) => db,
        Err(err) => panic!("Database error: {}.", err.to_string()),
    };

    let launch_result = rocket::build()
        .manage(db)
        .register("/api", catchers![not_found])
        .mount("/api", openapi_get_routes![index])
        .mount(
            "/api/swagger-ui",
            make_swagger_ui(&SwaggerUIConfig {
                url: "../openapi.json".to_owned(),
                ..Default::default()
            }),
        )
        .launch()
        .await;
    match launch_result {
        Ok(_) => println!("Shutdown successfully."),
        Err(err) => println!("Rocket had an error: {}.", err),
    }
}
