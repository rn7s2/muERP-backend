use sea_orm::{ConnectionTrait, Database, DatabaseConnection, DbBackend, DbErr, Statement};

pub async fn setup_db() -> Result<DatabaseConnection, DbErr> {
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
