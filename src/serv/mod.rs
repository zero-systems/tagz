use crate::{AppConfig, Connection};
use actix_web::{
    web::{self, Data},
    App, HttpServer,
};
use futures::lock::Mutex;
use routes::api as apis;
pub use service_error::ServiceError;

pub mod routes;
pub mod service_error;

#[inline]
pub async fn run(connection: Connection, cfg: AppConfig) -> std::io::Result<()> {
    let connection = Data::new(Mutex::new(connection));
    let cfg = Data::new(cfg);

    let http_target = cfg.http_target.clone(); // FIXME: remove?

    #[rustfmt::skip]
    HttpServer::new(move || {
        App::new()
            .app_data(Data::clone(&connection))
            .app_data(Data::clone(&cfg))
            .wrap(actix_web::middleware::Logger::default())
            .service(web::scope("api")
                .service(web::scope("v1")
                    .service(web::scope("tags")
                        .service(apis::tags::create)
                        .service(apis::tags::delete)
                        .service(apis::tags::list)
                    )
                    .service(web::scope("files")
                        .service(apis::files::create)
                        .service(apis::files::delete)
                        .service(apis::files::list)

                        .service(web::scope("{file_id}")
                            .service(apis::files::add)
                            .service(apis::files::remove)
                            )
                    ),
            ))
    })
    .bind(http_target.as_ref())?
    .run()
    .await
}
