use rusqlite::{Error, Row};

pub trait FromRow {
    fn from_row(row: &Row) -> Result<Self, Error>
    where
        Self: Sized;
}
