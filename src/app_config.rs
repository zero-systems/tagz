use clap::ArgMatches;

#[derive(Clone)]
pub struct AppConfig {
    pub http_target: Box<str>,
}

impl From<ArgMatches<'_>> for AppConfig {
    fn from(matches: ArgMatches) -> Self {
        Self {
            http_target: matches
                .value_of("http_target")
                .map(|x| x.into())
                .unwrap_or_else(|| "127.0.0.1:12345".into()),
        }
    }
}
