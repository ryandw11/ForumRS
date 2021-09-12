use sqlx::{Connection, MySqlConnection, Error, PgConnection, SqliteConnection, AnyConnection};
use crate::settings::{DatabaseType, MysqlSettings, PostgreSQLSettings, SqlSettings};

/// Manages the database connection for the website.
pub struct Database {
    connection: AnyConnection,
    conn_type: DatabaseType,
    in_db: bool
}

unsafe impl Send for Database{}

impl Database {
    /// Construct a new SQLite database connection from the provided settings.
    ///
    /// # Returns
    /// This returns either the Database struct or an SQLX error if one occurs.
    pub async fn new_sqlite(settings: &SqlSettings) -> Result<Self, Error> {
        let connection = AnyConnection::connect(&format!("sqlite://{}", settings.file_location)).await;
        if connection.is_err() {
            return Err(connection.unwrap_err());
        }

        Ok(Database {
            connection: connection.unwrap(),
            conn_type: DatabaseType::MySQL,
            in_db: true
        })
    }

    /// Construct a new MySQL database connection from the provided settings.
    /// This will use the database_name setting to connect to the database directly.
    ///
    /// # Returns
    /// This returns either the Database struct or an SQLX error if one occurs.
    pub async fn new_mysql(settings: &MysqlSettings) -> Result<Self, Error> {
        let connection = AnyConnection::connect(&format!("mysql://{}:{}@{}:{}/{}", settings.username, settings.password,
                                                           settings.url, settings.port, settings.database_name)).await;
        if connection.is_err() {
            return Err(connection.unwrap_err());
        }

        Ok(Database {
            connection: connection.unwrap(),
            conn_type: DatabaseType::MySQL,
            in_db: true
        })
    }

    /// Construct a new MySQL database connection from the provided settings.
    /// This will NOT directly connect to a database.
    ///
    /// # Returns
    /// This returns either the Database struct or an SQLX error if one occurs.
    pub async fn new_mysql_no_db(settings: &MysqlSettings) -> Result<Self, Error> {
        let connection = AnyConnection::connect(&format!("mysql://{}:{}@{}:{}", settings.username, settings.password,
                                                           settings.url, settings.port)).await;
        if connection.is_err() {
            return Err(connection.unwrap_err());
        }

        Ok(Database {
            connection: connection.unwrap(),
            conn_type: DatabaseType::MySQL,
            in_db: false
        })
    }

    /// Construct a new PostgreSQL database connection from the provided settings.
    /// This will directly connect to a database. The database name will automatically be converted
    /// to lowercase.
    ///
    /// # Returns
    /// This returns either the Database struct or an SQLX error if one occurs.
    pub async fn new_postgre(settings: &PostgreSQLSettings) -> Result<Self, Error> {
        let connection = AnyConnection::connect(&format!("postgresql://{}:{}@{}:{}/{}", settings.username, settings.password,
                                                        settings.url, settings.port, settings.database_name.to_lowercase())).await;
        if connection.is_err() {
            return Err(connection.unwrap_err());
        }

        Ok(Database {
            connection: connection.unwrap(),
            conn_type: DatabaseType::PostgreSQL,
            in_db: true
        })
    }

    /// Construct a new PostgreSQL database connection from the provided settings.
    /// This will NOT directly connect to a database.
    ///
    /// # Returns
    /// This returns either the Database struct or an SQLX error if one occurs.
    pub async fn new_postgre_no_db(settings: &PostgreSQLSettings) -> Result<Self, Error> {
        let connection = AnyConnection::connect(&format!("postgresql://{}:{}@{}:{}", settings.username, settings.password,
                                                        settings.url, settings.port)).await;
        if connection.is_err() {
            return Err(connection.unwrap_err());
        }

        Ok(Database {
            connection: connection.unwrap(),
            conn_type: DatabaseType::PostgreSQL,
            in_db: false
        })
    }

    pub fn is_in_db(&self) -> bool {
        self.in_db
    }

    pub fn get_type(&self) -> DatabaseType {
        self.conn_type.clone()
    }

    pub fn connection(&mut self) -> &mut AnyConnection {
        &mut self.connection
    }

    pub async fn close(self) {
        self.connection.close().await.unwrap();
    }
}