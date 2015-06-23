use std::sync::{RwLock};
use std::io::{Read, Write};

use mio::{EventLoop, Token, NonBlock, ReadHint, Handler};
use mio::buf::ByteBuf;
use mio::udp;

use super::stats::Stats;


const GOSSIP: Token = Token(0);


pub fn p2p_loop(stats: &RwLock<Stats>, host: &str, port: u16) {
    let server = udp::bind(&format!("{}:{}", host, port).parse().unwrap()
                            ).unwrap();
    let mut eloop = EventLoop::new().unwrap();
    eloop.register(&server, GOSSIP).unwrap();
    eloop.run(&mut Cantal {
        gossip: server,
    }).unwrap();
}

struct Cantal {
    gossip: NonBlock<udp::UdpSocket>,
}

impl Handler for Cantal {
    type Timeout = ();
    type Message = ();

    fn readable(&mut self, eloop: &mut EventLoop<Cantal>,
                tok: Token, hint: ReadHint)
    {
        match tok {
            GOSSIP => {
                let mut buf = ByteBuf::mut_with_capacity(4096);
                if let Ok(Some(addr)) = self.gossip.recv_from(&mut buf) {
                    let mut s = String::new();
                    buf.flip().read_to_string(&mut s);
                    println!("ADDR {:?} DATA {:?}", addr, s);
                }
            }
            _ => unreachable!(),
        }
    }
}
