use sqlx::SqlitePool;
use std::sync::OnceLock;

static DB_POOL: OnceLock<SqlitePool> = OnceLock::new();

pub async fn init_db_pool() {
    let db_path = crate::utils::get_nami_dir().join("sessions.db");
    let conn_str = format!("sqlite:{}?mode=rwc", db_path.to_string_lossy());
    let pool = SqlitePool::connect(&conn_str)
        .await
        .expect("Failed to create DB pool");
    DB_POOL.set(pool).expect("DB pool already initialized");
}

pub fn db_pool() -> &'static SqlitePool {
    DB_POOL.get().expect("DB pool not initialized. Call init_db_pool() first.")
}
