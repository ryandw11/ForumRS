use async_trait::async_trait;
use sqlx;
use sqlx::any::AnyDone;
use sqlx::Error;
use uuid::Uuid;

use crate::schema::database::Database;
use crate::settings::DatabaseType;

#[async_trait]
pub trait Table {
    async fn create(db: &mut Database) -> Result<AnyDone, Error>;
    async fn drop(db: &mut Database);
    async fn exists(db: &mut Database) -> bool;
}

/// The users table. This stores information about the user.
pub struct Users {}

impl Users {
    /// Insert a user into the table.
    pub async fn insert(db: &mut Database, uuid: Uuid, username: String, email: String, hashed_password: String, is_banned: bool, is_admin: bool) {
        let tp = db.get_type();
        match tp {
            DatabaseType::MySQL => {
                sqlx::query("INSERT INTO users (uuid, username, email, password, is_banned, is_admin) VALUES (?, ?, ?, ?, ?, ?)")
                    .bind(uuid.to_string())
                    .bind(username)
                    .bind(email)
                    .bind(hashed_password)
                    .bind(if is_banned {1} else {0})
                    .bind(if is_admin {1} else {0})
                    .execute(db.connection()).await.unwrap();
            }
            DatabaseType::SQLite => {
                sqlx::query("INSERT INTO users (uuid, username, email, password, is_banned, is_admin) VALUES (?, ?, ?, ?, ?, ?)")
                    .bind(uuid.to_string())
                    .bind(username)
                    .bind(email)
                    .bind(hashed_password)
                    .bind(if is_banned {1} else {0})
                    .bind(if is_admin {1} else {0})
                    .execute(db.connection()).await.unwrap();
            }
            DatabaseType::PostgreSQL => {
                sqlx::query("INSERT INTO users (uuid, username, email, password, is_banned, is_admin) VALUES ($1, $2, $3, $4, $5, $6);")
                    .bind(uuid.to_string())
                    .bind(username)
                    .bind(email)
                    .bind(hashed_password)
                    .bind(is_banned)
                    .bind(is_admin)
                    .execute(db.connection()).await.unwrap();
            }
        }
    }
}

#[async_trait]
impl Table for Users {
    async fn create(db: &mut Database) -> Result<AnyDone, Error> {
        let tp = db.get_type();
        match tp {
            DatabaseType::SQLite => {
                sqlx::query("CREATE TABLE IF NOT EXISTS users (\
                id INTEGER PRIMARY KEY,
                uuid VARCHAR(40) NOT NULL,\
                username VARCHAR(40) NOT NULL,\
                email VARCHAR(255) NOT NULL,\
                password VARCHAR(100) NOT NULL,\
                is_banned TINYINT NOT NULL,\
                is_admin TINYINT NOT NULL,\
                date INTEGER DEFAULT CURRENT_TIMESTAMP\
                )").execute(db.connection()).await
            }
            DatabaseType::MySQL => {
                sqlx::query("CREATE TABLE IF NOT EXISTS users (\
                id INTEGER PRIMARY KEY,
                uuid VARCHAR(40) NOT NULL,\
                username VARCHAR(40) NOT NULL,\
                email VARCHAR(255) NOT NULL,\
                password VARCHAR(100) NOT NULL,\
                is_banned BOOL NOT NULL,\
                is_admin BOOL NOT NULL,\
                date TIMESTAMP DEFAULT CURRENT_TIMESTAMP\
                )").execute(db.connection()).await
            }
            DatabaseType::PostgreSQL => {
                sqlx::query("CREATE TABLE IF NOT EXISTS users (\
                id SERIAL PRIMARY KEY,
                uuid VARCHAR(40) NOT NULL,\
                username VARCHAR(40) NOT NULL,\
                email VARCHAR(255) NOT NULL,\
                password VARCHAR(100) NOT NULL,\
                is_banned BOOL NOT NULL,\
                is_admin BOOL NOT NULL,\
                date TIMESTAMP DEFAULT CURRENT_TIMESTAMP\
                )").execute(db.connection()).await
            }
        }
    }

    async fn drop(db: &mut Database) {
        sqlx::query("DROP TABLE users").execute(db.connection()).await.unwrap();
    }

    async fn exists(db: &mut Database) -> bool {
        todo!()
    }
}

/// The table for user sessions.
/// This stores active login sessions.
pub struct Sessions {}

#[async_trait]
impl Table for Sessions {
    async fn create(db: &mut Database) -> Result<AnyDone, Error> {
        let tp = db.get_type();
        match tp {
            DatabaseType::SQLite => {
                sqlx::query("CREATE TABLE IF NOT EXISTS sessions (\
                id INTEGER PRIMARY KEY,
                session_uuid VARCHAR(40) NOT NULL,\
                user_uuid VARCHAR(40) NOT NULL,\
                session_creation BIGINT\
                )").execute(db.connection()).await
            }
            DatabaseType::MySQL => {
                sqlx::query("CREATE TABLE IF NOT EXISTS sessions (\
                id INTEGER PRIMARY KEY,
                session_uuid VARCHAR(40) NOT NULL,\
                user_uuid VARCHAR(40) NOT NULL,\
                session_creation BIGINT\
                )").execute(db.connection()).await
            }
            DatabaseType::PostgreSQL => {
                sqlx::query("CREATE TABLE IF NOT EXISTS sessions (\
                id SERIAL PRIMARY KEY,
                session_uuid VARCHAR(40) NOT NULL,\
                user_uuid VARCHAR(40) NOT NULL,\
                session_creation BIGINT\
                );").execute(db.connection()).await
            }
        }
    }

    async fn drop(db: &mut Database) {
        sqlx::query("DROP TABLE sessions").execute(db.connection()).await.unwrap();
    }

    async fn exists(db: &mut Database) -> bool {
        todo!()
    }
}

/// The table to store thread data.
pub struct Threads {}

#[async_trait]
impl Table for Threads {
    async fn create(db: &mut Database) -> Result<AnyDone, Error> {
        let tp = db.get_type();
        match tp {
            DatabaseType::SQLite => {
                sqlx::query("CREATE TABLE IF NOT EXISTS threads (\
                id INTEGER PRIMARY KEY,
                uuid VARCHAR(40) NOT NULL,\
                forum_uuid VARCHAR(40) NOT NULL,\
                name VARCHAR(100) NOT NULL,\
                content TEXT NOT NULL,\
                creator VARCHAR(40) NOT NULL,\
                locked TINYINT NOT NULL,\
                date INTEGER DEFAULT CURRENT_TIMESTAMP\
                )").execute(db.connection()).await
            }
            DatabaseType::MySQL => {
                sqlx::query("CREATE TABLE IF NOT EXISTS threads (\
                id INTEGER PRIMARY KEY,
                uuid VARCHAR(40) NOT NULL,\
                forum_uuid VARCHAR(40) NOT NULL,\
                name VARCHAR(100) NOT NULL,\
                content TEXT NOT NULL,\
                creator VARCHAR(40) NOT NULL,\
                locked TINYINT NOT NULL,\
                date TIMESTAMP DEFAULT CURRENT_TIMESTAMP\
                )").execute(db.connection()).await
            }
            DatabaseType::PostgreSQL => {
                sqlx::query("CREATE TABLE IF NOT EXISTS threads (\
                id SERIAL PRIMARY KEY,
                uuid VARCHAR(40) NOT NULL,\
                forum_uuid VARCHAR(40) NOT NULL,\
                name VARCHAR(100) NOT NULL,\
                content TEXT NOT NULL,\
                creator VARCHAR(40) NOT NULL,\
                locked SMALLINT NOT NULL,\
                date TIMESTAMP DEFAULT CURRENT_TIMESTAMP\
                )").execute(db.connection()).await
            }
        }
    }

    async fn drop(db: &mut Database) {
        sqlx::query("DROP TABLE threads").execute(db.connection()).await.unwrap();
    }

    async fn exists(db: &mut Database) -> bool {
        todo!()
    }
}

pub struct Posts {}

#[async_trait]
impl Table for Posts {
    async fn create(db: &mut Database) -> Result<AnyDone, Error> {
        let tp = db.get_type();
        match tp {
            DatabaseType::SQLite => {
                sqlx::query("CREATE TABLE IF NOT EXISTS posts (\
                id INTEGER PRIMARY KEY,
                uuid VARCHAR(40) NOT NULL,\
                thread_uuid VARCHAR(40) NOT NULL,\
                creator VARCHAR(40) NOT NULL,\
                content TEXT NOT NULL,\
                date INTEGER DEFAULT CURRENT_TIMESTAMP\
                )").execute(db.connection()).await
            }
            DatabaseType::MySQL => {
                sqlx::query("CREATE TABLE IF NOT EXISTS posts (\
                id INTEGER PRIMARY KEY,
                uuid VARCHAR(40) NOT NULL,\
                thread_uuid VARCHAR(40) NOT NULL,\
                creator VARCHAR(40) NOT NULL,\
                content TEXT NOT NULL,\
                date TIMESTAMP DEFAULT CURRENT_TIMESTAMP\
                )").execute(db.connection()).await
            }
            DatabaseType::PostgreSQL => {
                sqlx::query("CREATE TABLE IF NOT EXISTS posts (\
                id SERIAL PRIMARY KEY,
                uuid VARCHAR(40) NOT NULL,\
                thread_uuid VARCHAR(40) NOT NULL,\
                creator VARCHAR(40) NOT NULL,\
                content TEXT NOT NULL,\
                date TIMESTAMP DEFAULT CURRENT_TIMESTAMP\
                )").execute(db.connection()).await
            }
        }
    }

    async fn drop(db: &mut Database) {
        sqlx::query("DROP TABLE posts").execute(db.connection()).await.unwrap();
    }

    async fn exists(db: &mut Database) -> bool {
        todo!()
    }
}

pub struct Forums {}

impl Forums {
    pub async fn insert(db: &mut Database, uuid: Uuid, name: String, caption: String) {
        let tp = db.get_type();
        match tp {
            DatabaseType::MySQL | DatabaseType::SQLite => {
                sqlx::query("INSERT INTO forums (uuid, name, caption) VALUES (?, ?, ?);")
                    .bind(uuid.to_string())
                    .bind(name)
                    .bind(caption)
                    .execute(db.connection()).await.unwrap();
            }
            DatabaseType::PostgreSQL => {
                sqlx::query("INSERT INTO forums (uuid, name, caption) VALUES ($1, $2, $3);")
                    .bind(uuid.to_string())
                    .bind(name)
                    .bind(caption)
                    .execute(db.connection()).await.unwrap();
            }
        }
    }
}

#[async_trait]
impl Table for Forums {
    async fn create(db: &mut Database) -> Result<AnyDone, Error> {
        let tp = db.get_type();
        match tp {
            DatabaseType::SQLite => {
                sqlx::query("CREATE TABLE IF NOT EXISTS forums (\
                id INTEGER PRIMARY KEY,
                uuid VARCHAR(40) NOT NULL,\
                name VARCHAR(100) NOT NULL,\
                caption TINYTEXT NOT NULL,\
                date INTEGER DEFAULT CURRENT_TIMESTAMP\
                )").execute(db.connection()).await
            }
            DatabaseType::MySQL => {
                sqlx::query("CREATE TABLE IF NOT EXISTS forums (\
                id INTEGER PRIMARY KEY,
                uuid VARCHAR(40) NOT NULL,\
                name VARCHAR(100) NOT NULL,\
                caption TINYTEXT NOT NULL,\
                date TIMESTAMP DEFAULT CURRENT_TIMESTAMP\
                )").execute(db.connection()).await
            }
            DatabaseType::PostgreSQL => {
                sqlx::query("CREATE TABLE IF NOT EXISTS forums (\
                id SERIAL PRIMARY KEY,
                uuid VARCHAR(40) NOT NULL,\
                name VARCHAR(100) NOT NULL,\
                caption VARCHAR(255) NOT NULL,\
                date TIMESTAMP DEFAULT CURRENT_TIMESTAMP\
                )").execute(db.connection()).await
            }
        }
    }

    async fn drop(db: &mut Database) {
        sqlx::query("DROP TABLE forums").execute(db.connection()).await.unwrap();
    }

    async fn exists(db: &mut Database) -> bool {
        todo!()
    }
}

pub struct BannedUsers {}

#[async_trait]
impl Table for BannedUsers {
    async fn create(db: &mut Database) -> Result<AnyDone, Error> {
        let tp = db.get_type();
        match tp {
            DatabaseType::SQLite => {
                sqlx::query("CREATE TABLE IF NOT EXISTS banned_users (\
                id INTEGER PRIMARY KEY,
                user_uuid VARCHAR(40) NOT NULL,\
                ban_date INTEGER NOT NULL,\
                unban_date INTEGER NOT NULL,\
                date INTEGER DEFAULT CURRENT_TIMESTAMP\
                )").execute(db.connection()).await
            }
            DatabaseType::MySQL => {
                sqlx::query("CREATE TABLE IF NOT EXISTS banned_users (\
                id INTEGER PRIMARY KEY,
                user_uuid VARCHAR(40) NOT NULL,\
                ban_date INTEGER NOT NULL,\
                unban_date INTEGER NOT NULL,\
                date TIMESTAMP DEFAULT CURRENT_TIMESTAMP\
                )").execute(db.connection()).await
            }
            DatabaseType::PostgreSQL => {
                sqlx::query("CREATE TABLE IF NOT EXISTS banned_users (\
                id SERIAL PRIMARY KEY,
                user_uuid VARCHAR(40) NOT NULL,\
                ban_date INTEGER NOT NULL,\
                unban_date INTEGER NOT NULL,\
                date TIMESTAMP DEFAULT CURRENT_TIMESTAMP\
                )").execute(db.connection()).await
            }
        }
    }

    async fn drop(db: &mut Database) {
        sqlx::query("DROP TABLE banned_users").execute(db.connection()).await.unwrap();
    }

    async fn exists(db: &mut Database) -> bool {
        todo!()
    }
}