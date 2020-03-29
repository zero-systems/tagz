#[macro_export]
macro_rules! json {
    ($code:ident, $obj:expr) => {
        Ok(
            actix_web::HttpResponse::build(actix_web::http::StatusCode::$code)
                .json(serde_json::json!({ "ok": $obj })),
        )
    };

    ($obj:expr) => {
        Ok(
            actix_web::HttpResponse::build(actix_web::http::StatusCode::OK)
                .json(serde_json::json!({ "ok": $obj })),
        );
    };
}

#[macro_export]
macro_rules! no_content {
    () => {
        Ok(actix_web::HttpResponse::build(
            actix_web::http::StatusCode::NO_CONTENT,
        ))
    };
}

#[macro_export]
macro_rules! last_inserted {
    ($conn:expr, $table:literal) => {
        $conn.query_row(
            concat!(
                "SELECT * FROM `",
                $table,
                "` WHERE `id`=last_insert_rowid()"
            ),
            params! {},
            Self::from_row,
        )
    };
}
