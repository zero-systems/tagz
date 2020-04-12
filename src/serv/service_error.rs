use actix_web::{dev::Body, http::StatusCode, HttpResponse, ResponseError};
use rusqlite::Error as SqlError;
use serde::ser::Serialize;
use serde_json::value::Value;
use std::{
    borrow::Cow,
    fmt::{Debug, Display, Formatter, Result as FmtResult},
};

#[derive(serde::Serialize, Debug)]
pub struct ServiceError {
    #[serde(skip)]
    status_code: StatusCode,

    status: &'static str,
    message: Cow<'static, str>,

    details: Option<Value>, // IDEA: maybe Box<dyn Serialize> ?
}

impl ServiceError {
    pub fn bad_request<C>(status: &'static str, message: C) -> Self
    where
        C: Into<Cow<'static, str>>,
    {
        Self {
            status_code: StatusCode::BAD_REQUEST,
            status,
            message: message.into(),
            details: None,
        }
    }

    pub fn not_found<C>(status: &'static str, message: C) -> Self
    where
        C: Into<Cow<'static, str>>,
    {
        Self {
            status_code: StatusCode::NOT_FOUND,
            status,
            message: message.into(),
            details: None,
        }
    }

    pub fn with_details<S>(mut self, details: S) -> Self
    where
        S: Serialize,
    {
        self.details = Some(serde_json::json!(details));
        self
    }
}

impl Display for ServiceError {
    #[inline]
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        Debug::fmt(self, fmt)
    }
}

impl From<SqlError> for ServiceError {
    fn from(err: SqlError) -> Self {
        Self {
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
            status: "SQL",
            message: Cow::Owned(format!("{}", err)),
            details: None,
        }
    }
}

impl ResponseError for ServiceError {
    #[inline]
    fn status_code(&self) -> StatusCode {
        self.status_code
    }

    fn error_response(&self) -> HttpResponse<Body> {
        #[derive(serde::Serialize)]
        struct ErrorBody<'a> {
            err: &'a ServiceError,
        }

        HttpResponse::build(self.status_code).json(ErrorBody { err: self })
    }
}
