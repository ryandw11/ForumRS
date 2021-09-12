use crate::settings::{DatabaseType, SettingsManager, SqlSettings, MysqlSettings, PostgreSQLSettings};
use crate::schema::database::Database;
use sqlx::{Connection, AnyConnection};
use crate::schema::tables::{Users, Table, Sessions, Threads, Posts, Forums, BannedUsers};
use uuid::Uuid;

pub async fn create_schema_mysql(mysql_settings: &MysqlSettings) {
    let mut db = Database::new_mysql_no_db(mysql_settings).await.unwrap();
    sqlx::query(&format!("CREATE DATABASE IF NOT EXISTS {}", mysql_settings.database_name))
        .execute(db.connection()).await.unwrap();
    db.close().await;
}

pub async fn create_schema_postgre(postgre_settings: &PostgreSQLSettings) {
    let mut db = Database::new_postgre_no_db(postgre_settings).await.unwrap();
    sqlx::query(&format!("CREATE DATABASE {};", postgre_settings.database_name))
        .execute(db.connection()).await.unwrap();
    println!("Test");
    db.close().await;
}

/// Setup the database.
/// Note: The actual database/schema must be already created.
pub async fn setup_database(db: &mut Database) {
    let conn: &mut AnyConnection = db.connection();

    // Create the users table.
    Users::create(db).await.unwrap();
    Sessions::create(db).await.unwrap();
    Threads::create(db).await.unwrap();
    Posts::create(db).await.unwrap();
    Forums::create(db).await.unwrap();
    BannedUsers::create(db).await.unwrap();
}