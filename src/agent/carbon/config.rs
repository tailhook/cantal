use quire::validate::*;


#[derive(Debug, RustcDecodable, Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
}

pub fn validator<'x>() -> Structure<'x> {
    Structure::new()
    .member("host", Scalar::new())
    .member("port", Scalar::new())
}
