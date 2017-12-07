use std::path::Path;
use tk_http::server::{Head};

#[derive(Clone, Debug)]
pub enum Format {
    Json,
    Cbor,
}

#[derive(Clone, Debug)]
pub enum RemoteRoute {
    QueryByHost,  // POST
    MemInfo,
}

#[derive(Clone, Debug)]
pub enum Route {
    Index,
    Static(String),
    NotFound,
    WebSocket,
    Status(Format),
    AllProcesses(Format),
    AllSockets(Format),
    AllMetrics(Format),
    AllPeers(Format),
    PeersWithRemote(Format),
    RemoteStats(Format),
    StartRemote(Format),  // POST
    Query(Format),        // POST
    Remote(RemoteRoute, Format),
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

fn validate_path<P: AsRef<Path>>(path: P) -> bool {
    use std::path::Component::Normal;
    for cmp in Path::new(path.as_ref()).components(){
        match cmp {
            Normal(_) => {}
            _ => return false,
        }
    }
    return true;
}


pub fn route(head: &Head) -> Route {
    use self::Route::*;
    use self::RemoteRoute::*;
    use self::Format::*;
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
        ("css", _) | ("js", _) | ("fonts", _) => {
            let path = path.trim_left_matches('/');
            if !validate_path(path) {
                // TODO(tailhook) implement 400
                NotFound
            } else {
                Static(path.to_string())
            }
        }
        ("ws", _) => WebSocket,
        ("status.json", "") => Status(Json),
        ("all_processes.json", "") => AllProcesses(Json),
        ("all_sockets.json", "") => AllSockets(Json),
        ("all_metrics.json", "") => AllMetrics(Json),
        ("all_peers.json", "") => AllPeers(Json),
        ("peers_with_remote.json", "") => PeersWithRemote(Json),
        ("remote_stats.json", "") => RemoteStats(Json),
        ("start_remote.json", "") => StartRemote(Json),
        ("query.cbor", "") => Query(Cbor),
        ("remote", "query_by_host.cbor") => Remote(QueryByHost, Cbor),
        ("remote", "mem_info.json") => Remote(MemInfo, Json),
        (_, _) => Index,
    };
    debug!("Routed {:?} to {:?}", path, route);
    route
}
