mod controllers;
mod entities;
mod services;

use controllers::item;
use rocket::{
    catch, catchers,
    serde::json::{json, Value},
};
use rocket_okapi::{
    openapi_get_routes,
    swagger_ui::{make_swagger_ui, SwaggerUIConfig},
};
use services::db::setup_db;

#[catch(404)]
fn not_found() -> Value {
    json!({
        "status": "error",
        "reason": "Resource was not found."
    })
}

#[rocket::main]
async fn main() {
    let db = match setup_db().await {
        Ok(db) => db,
        Err(err) => panic!("Database error: {}.", err.to_string()),
    };

    let launch_result = rocket::build()
        .manage(db)
        .register("/", catchers![not_found])
        .mount("/api", openapi_get_routes![item::index])
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
