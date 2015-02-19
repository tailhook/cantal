use std::sync::RwLock;
use std::time::Duration;
use serialize::json::Json;
use serialize::json;

use super::aio;
use super::stats::Stats;
use super::aio::http;
use super::staticfiles;


fn handle_request(stats: &RwLock<Stats>, req: &http::Request)
    -> Result<http::Response, http::Error>
{
    if  req.uri().starts_with("/js") ||
        req.uri().starts_with("/css/") ||
        req.uri().starts_with("/fonts/") ||
        req.uri() == "/"
    {
        return staticfiles::serve(req);
    } else {
        let stats = stats.read().unwrap();
        let mut builder = http::ResponseBuilder::new(req, http::Status::Ok);
        builder.set_body(format!("{}", Json::Object(vec!(
                ("startup_time".to_string(), Json::U64(stats.startup_time))
                ).into_iter().collect())
            ).into_bytes());
        Ok(builder.take())
    }
}


fn read_stats() {
}


pub fn run_server<'x>(stats: &RwLock<Stats>, host: String, port: u16)
    -> Result<(), String>
{
    let handler: &for<'b> Fn(&'b aio::http::Request<'b>)
        -> Result<aio::http::Response, aio::http::Error>
        = &|&:req| {
        handle_request(stats, req)
    };
    let mut main = try!(aio::MainLoop::new()
        .map_err(|e| format!("Can't create main loop: {}", e)));
    try!(main.add_http_server(host.as_slice(), port, handler)
        .map_err(|e| format!("Can't bind {}:{}: {}", host, port, e)));
    main.add_interval(Duration::seconds(2),read_stats);
    main.run();
}
