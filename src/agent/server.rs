use std::time::Duration;

use super::aio;
use super::aio::http;


fn http_handler(req: &http::Request) {
    println!("REQ {:?}", req);
}


fn read_stats() {
}


pub fn run_server(host: String, port: u16) -> Result<(), String> {
    let mut main = try!(aio::MainLoop::new()
        .map_err(|e| format!("Can't create main loop: {}", e)));
    try!(main.add_http_server(host.as_slice(), port, http_handler)
        .map_err(|e| format!("Can't bind {}:{}: {}", host, port, e)));
    main.add_interval(Duration::seconds(2),read_stats);
    main.run();
}
