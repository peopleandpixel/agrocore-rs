use agrocore_shared::SharedError;
use sqlx::postgres::PgDatabaseError;

pub fn map_db_error(e: sqlx::Error) -> SharedError {
    #[allow(clippy::collapsible_if)]
    if let Some(pg_err) = e.as_database_error() {
        if let Some(db_err) = pg_err.try_downcast_ref::<PgDatabaseError>() {
            return match db_err.code() {
                "23505" => SharedError::AlreadyExists(db_err.message().to_string()),
                "23503" => SharedError::ReferenceError(db_err.message().to_string()),
                _ => SharedError::Database(e.to_string()),
            };
        }
    }
    match e {
        sqlx::Error::RowNotFound => SharedError::NotFound("Row not found".into()),
        _ => SharedError::Database(e.to_string()),
    }
}
