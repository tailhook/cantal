use quire::validate::*;


#[derive(Debug, RustcDecodable, Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub interval: u32,
    pub enable_cgroup_stats: bool,
    pub enable_application_metrics: bool,
}

pub fn validator<'x>() -> Structure<'x> {
    Structure::new()
    .member("host", Scalar::new())
    .member("port", Scalar::new().default(2003))
    .member("interval",
        Numeric::new().min(1).max(86400).default(10))
    .member("enable_cgroup_stats", Scalar::new().default(false))
    .member("enable_application_metrics", Scalar::new().default(false))
}
