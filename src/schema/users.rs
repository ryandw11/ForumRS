use sqlx::{MySql, Connection};

trait UserTableTrait {
    fn create_table(conn: impl Connection) -> Result<(), String>;
}

struct UserTable<T> {}

impl UserTableTrait for UserTable<MySql> {
    fn create_table(conn: impl Connection) -> Result<(), String> {
        sqlx::query("CREATE TABLE")
    }
}