mod file;
mod tag;

// FIXME: remake structure
mod serv_prelude {
    pub use crate::serv::{routes::Result, ServiceError};
    pub use actix_web::Responder;
}

use crate::{Connection, FromRow, SqlResult};
use chrono::NaiveDateTime;
use rusqlite::{params, OptionalExtension, ToSql};
use tagz_cg_from_row::FromRow;

pub use file::File;
pub use tag::Tag;
