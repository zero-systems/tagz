#[macro_use]
extern crate log;
extern crate tagz;

use std::error::Error;
use tagz::AppConfig;

#[actix_rt::main]
async fn main() -> Result<(), Box<dyn Error>> {
    std::env::set_var("RUST_LOG", "tagzd=info,actix_web=info");
    env_logger::init();

    let matches = {
        use clap::{App, Arg};

        App::new("tagzd")
            .about("Daemon for TagZ.")
            .author("Øsystems")
            .arg(
                Arg::with_name("http_target")
                    .help("Http target (default: 127.0.0.1:12345)")
                    .short("t")
                    .long("target")
                    .takes_value(true),
            )
            .get_matches()
    };

    let cfg = AppConfig::from(matches);
    let connection = tagz::get_conn(std::path::Path::new("tagz.db"))?;

    info!("Connection to db file is set.");

    tagz::serv::run(connection, cfg).await?;

    Ok(())
}
