use std::io::{self, BufReader, BufRead};
use std::fs::File;
use std::net::{SocketAddrV4, Ipv4Addr};
use std::collections::{HashMap, HashSet};
use rustc_serialize::{Encoder, Encodable};


const MAX_CONNECTION_DETAILS: usize = 1000;


#[derive(RustcEncodable, Debug)]
pub struct Stats {
    pub by_state: HashMap<State, usize>,
    pub rx_queue: usize,
    pub tx_queue: usize,
}

#[derive(RustcEncodable, Debug)]
pub struct Passive {
    pub stats: Stats,
    pub listeners: Vec<Socket>,
    pub clients: Option<Vec<Socket>>,
}

#[derive(RustcEncodable, Debug)]
pub struct Active {
    pub stats: Stats,
    pub connections: Option<Vec<Socket>>,
}

pub trait MyEncodable {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error>;
}

impl MyEncodable for SocketAddrV4 {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        s.emit_str(&format!("{}", self))
    }
}

#[derive(RustcEncodable, Debug, Clone)]
pub struct Socket {
    pub local_address: SocketAddrV4,
    pub remote_address: SocketAddrV4,
    pub state: State,
    pub tx_queue: usize,
    pub rx_queue: usize,
    pub uid: u32,
}

#[allow(dead_code, non_camel_case_types)]
#[derive(Clone, Copy, Debug, RustcEncodable, Hash, PartialEq, Eq)]
pub enum State {
    UNKNOWN = 0,
    ESTABLISHED,
    SYN_SENT,
    SYN_RECV,
    FIN_WAIT1,
    FIN_WAIT2,
    TIME_WAIT,
    CLOSE,
    CLOSE_WAIT,
    LAST_ACK,
    LISTEN,
    CLOSING,
}

#[derive(RustcEncodable, Debug)]
pub struct Connections {
    pub global: Stats,
    pub by_user: HashMap<u32, Stats>,
    pub passive: HashMap<u32, HashMap<u16, Passive>>,
    pub active: HashMap<u32, HashMap<u16, Active>>,
}

impl Stats {
    pub fn new() -> Stats {
        Stats {
            by_state: HashMap::new(),
            rx_queue: 0,
            tx_queue: 0,
        }
    }
    pub fn add(&mut self, sock: &Socket) {
        *self.by_state.entry(sock.state).or_insert(0) += 1;
        self.tx_queue += sock.tx_queue;
        self.rx_queue += sock.rx_queue;
    }
}

impl From<u8> for State {
    fn from(x: u8) -> State {
        use self::State::*;
        match x {
            1 => ESTABLISHED,
            2 => SYN_SENT,
            3 => SYN_RECV,
            4 => FIN_WAIT1,
            5 => FIN_WAIT2,
            6 => TIME_WAIT,
            7 => CLOSE,
            8 => CLOSE_WAIT,
            9 => LAST_ACK,
            10 => LISTEN,
            11 => CLOSING,
            _ => UNKNOWN,
        }
    }
}

impl Active {
    fn new() -> Active {
        Active {
            stats: Stats::new(),
            connections: Some(Vec::new()),
        }
    }
    fn add(&mut self, sock: Socket) {
        self.stats.add(&sock);
        self.connections.as_mut().map(|vec| vec.push(sock));
        let len = self.connections.as_ref().map(|x| x.len()).unwrap_or(0);
        if len > MAX_CONNECTION_DETAILS {
            self.connections = None
        }
    }
}

impl Passive {
    fn new() -> Passive {
        Passive {
            stats: Stats::new(),
            listeners: Vec::new(),
            clients: Some(Vec::new()),
        }
    }
    fn add(&mut self, sock: Socket) {
        if sock.state == State::LISTEN {
            self.listeners.push(sock);
        } else {
            self.stats.add(&sock);
            self.clients.as_mut().map(|vec| vec.push(sock));
            let len = self.clients.as_ref().map(|x| x.len()).unwrap_or(0);
            if len > MAX_CONNECTION_DETAILS {
                self.clients = None
            }
        }
    }
}

/// Parses socket addr defined in /proc/net/tcp
/// It hexadecimal ip:port so it's fixed size
fn parse_addr(val: &str) -> Option<SocketAddrV4> {
    if val.len() != 13 {
        return None;
    }
    let port = match u16::from_str_radix(&val[9..13], 16) {
        Ok(x) => x,
        Err(..) => return None,
    };
    let ip = match u32::from_str_radix(&val[..8], 16) {
        Ok(x) => x,
        Err(..) => return None,
    };
    Some(SocketAddrV4::new(Ipv4Addr::from(ip), port))
}

fn parse_line(line: &str) -> Socket {
    let mut pieces = line.split_whitespace();
    pieces.next(); // Skip slot number
    let local = pieces.next().and_then(parse_addr)
        .unwrap_or(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 0));
    let remote = pieces.next().and_then(parse_addr)
        .unwrap_or(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 0));
    let status = pieces.next()
        .and_then(|x| u8::from_str_radix(x, 16).ok())
        .unwrap_or(0);
    let mut queues = pieces.next().unwrap_or("0:0").split(':');
    let tx = queues.next()
        .and_then(|x| usize::from_str_radix(x, 16).ok())
        .unwrap_or(0);
    let rx = queues.next()
        .and_then(|x| usize::from_str_radix(x, 16).ok())
        .unwrap_or(0);
    let uid = pieces.skip(2).next()
        .and_then(|x| x.parse::<u32>().ok())
        .unwrap_or(0);
    return Socket {
        local_address: local,
        remote_address: remote,
        state: status.into(),
        tx_queue: tx,
        rx_queue: rx,
        uid: uid,
    }
}

pub fn read() -> Option<Connections> {
    _read().map_err(|e| error!("Can't read tcp connections: {}", e)).ok()
}

fn _read() -> io::Result<Connections> {
    let mut line = String::with_capacity(200);
    let mut file = BufReader::new(try!(File::open("/proc/net/tcp")));
    try!(file.read_line(&mut line)); // header
    let mut stats = Stats::new();
    let mut by_user = HashMap::new();
    // Passive are accepted connections on listening socket
    let mut passive = HashMap::new();
    // Active are connections that are established to a remote host
    let mut active = HashMap::new();
    // Set of addresses that are already known to be passive
    let mut passive_addr = HashSet::new();
    // Set of addresses that are already known to be active
    let mut active_addr = HashSet::new();
    // Sockets that are unknown to be active or passive
    // If we see a duplicate address at either side we know that the
    // addresss is use for active or passive socket for sure.
    let mut local_unknown = HashMap::<SocketAddrV4, Socket>::new();
    let mut remote_unknown = HashMap::<SocketAddrV4, Socket>::new();
    loop {
        line.clear();
        try!(file.read_line(&mut line));
        if line.len() == 0 { break; }
        let sock = parse_line(&line);
        stats.add(&sock);
        by_user.entry(sock.uid).or_insert_with(Stats::new).add(&sock);

        let la = sock.local_address;
        let ra = sock.remote_address;
        if passive_addr.contains(&la) {
            passive.entry(sock.uid).or_insert_with(HashMap::new)
                .entry(la.port()).or_insert_with(Passive::new).add(sock);
        } else if active_addr.contains(&ra) {
            active.entry(sock.uid).or_insert_with(HashMap::new)
                .entry(ra.port()).or_insert_with(Active::new).add(sock);
        } else if let Some(oldsock) = local_unknown.remove(&la) {
            remote_unknown.remove(&oldsock.remote_address);
            passive_addr.insert(la);
            passive.entry(oldsock.uid).or_insert_with(HashMap::new)
                .entry(la.port()).or_insert_with(Passive::new).add(oldsock);
            passive.entry(sock.uid).or_insert_with(HashMap::new)
                .entry(la.port()).or_insert_with(Passive::new).add(sock)
        } else if let Some(oldsock) = remote_unknown.remove(&ra) {
            local_unknown.remove(&oldsock.local_address);
            active_addr.insert(ra);
            active.entry(oldsock.uid).or_insert_with(HashMap::new)
                .entry(ra.port()).or_insert_with(Active::new).add(oldsock);
            active.entry(sock.uid).or_insert_with(HashMap::new)
                .entry(ra.port()).or_insert_with(Active::new).add(sock);
        } else if sock.state == State::LISTEN {
            passive_addr.insert(la);
            passive.entry(sock.uid).or_insert_with(HashMap::new)
                .entry(la.port()).or_insert_with(Passive::new).add(sock);
        } else {
            local_unknown.insert(la, sock.clone());
            remote_unknown.insert(ra, sock);
        }
    }
    // All sockets which don't have any other socket on the same address are
    // assumed to be active. I.e. in normal circumstances, you will have
    // at least LISTEN kind of socket as a pair for connection. Unless
    // listening socket is already closed, in the latter case it shouldn't
    // pose any resource issues, so don't care little inaccuracy
    // TODO(tailhook) figure out whether sockets like `0.0.0.0` work fine.
    //
    // Unknown sockets are duplicated both in local and remote list so we
    // just traverse one of them:
    for (_, sock) in remote_unknown.into_iter() {
        active.entry(sock.uid).or_insert_with(HashMap::new)
            .entry(sock.remote_address.port())
            .or_insert_with(Active::new).add(sock);
    }

    Ok(Connections {
        global: stats,
        by_user: by_user,
        active: active,
        passive: passive,
    })
}
