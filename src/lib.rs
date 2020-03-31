#![feature(stmt_expr_attributes)]

#[macro_use]
extern crate lazy_static;

pub use app_config::AppConfig;
pub use from_row::FromRow;
pub use rusqlite::{Connection, Error as SqlError, Result as SqlResult};
use std::path::Path;

mod app_config;
mod config;
mod from_row;
pub mod models;
pub mod serv;

#[inline]
pub fn get_conn(path: &Path) -> SqlResult<Connection> {
    // check if file exists
    let exists = path.exists();

    let connection = Connection::open(path)?;
    rusqlite::vtab::array::load_module(&connection).unwrap();

    if !exists {
        prepare_tables(&connection)?;
    }

    Ok(connection)
}

#[inline]
pub fn prepare_tables(conn: &Connection) -> SqlResult<()> {
    for table in config::TABLES {
        conn.execute(table, rusqlite::params! {})?;
    }

    Ok(())
}
