use super::ServiceError;
use crate::Connection;
use actix_web::{delete, get, post, web, Responder};
use futures::lock::Mutex;
use serde::Deserialize;

pub mod api;

pub type ConnLock = web::Data<Mutex<Connection>>;
pub type Result<T> = std::result::Result<T, ServiceError>;
