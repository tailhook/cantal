use tk_http::server::{Head};

#[derive(Clone, Debug)]
pub enum Route {
    Index,
    Static(String),
    NotFound,
}

fn path_component(path: &str) -> (&str, &str) {
    let path = if path.starts_with('/') {
        &path[1..]
    } else {
        path
    };
    match path.bytes().position(|x| x == b'/') {
        Some(end) => (&path[..end], &path[end+1..]),
        None => {
            let end = path.bytes().position(|x| x == b'.')
                .unwrap_or(path.as_bytes().len());
            (&path[..end], "")
        }
    }
}

pub fn route(head: &Head) -> Route {
    use self::Route::*;
    let path = if let Some(path) = head.path() {
        path
    } else {
        return Route::NotFound;
    };
    let path = match path.find('?') {
        Some(x) => &path[..x],
        None => path,
    };
    let route = match path_component(&path[..]) {
        ("", _) => Index,
        ("css", _) => Static(path.to_string()),
        ("js", _) => Static(path.to_string()),
        ("fonts", _) => Static(path.to_string()),
        (_, _) => Index,
    };
    debug!("Routed {:?} to {:?}", path, route);
    route
}
