use std::str::FromStr;
use std::fs::File;
use std::path::Path;
use std::io::{BufReader, Read, BufRead};
use std::collections::HashMap;
use std::collections::hash_map::Entry::{Vacant,Occupied};

use cantal::itertools::{words};
use cantal::Value::{Counter, Integer};

use super::Tip;
use super::super::stats::Key;


pub fn read(t: &mut Tip) -> Option<u64> {
    let mut boot_time = None::<u64>;
    File::open(&Path::new("/proc/uptime"))
        .and_then(|mut f| {
            let mut buf = String::with_capacity(100);
            f.read_to_string(&mut buf)
            .map(|_| buf)})
        .map(|buf| {
            let mut pieces = words(&buf);
            // TODO(tailhook) they are float counters?
            t.add_next_float(Key::metric("uptime"), &mut pieces);
            t.add_next_float(Key::metric("idle_time"), &mut pieces);
        }).ok();
    File::open(&Path::new("/proc/loadavg"))
        .and_then(|mut f| {
            let mut buf = String::with_capacity(100);
            f.read_to_string(&mut buf)
            .map(|_| buf)
        })
        .map(|buf| {
            let mut pieces = words(&buf);
            t.add_next_float(Key::metric("load_avg_1min"), &mut pieces);
            t.add_next_float(Key::metric("load_avg_5min"), &mut pieces);
            t.add_next_float(Key::metric("load_avg_15min"), &mut pieces);
            pieces.next()
                .map(|x| x.splitn(1, '/'))
                .map(|mut x| {
                    t.add_next_cnt(Key::metric("proc_runnable"), &mut x);
                    t.add_next_cnt(Key::metric("proc_total"), &mut x);
                });
            t.add_next_float(Key::metric("last_pid"), &mut pieces);
        }).ok();
    File::open(&Path::new("/proc/stat")).and_then(|f| {
        let mut f = BufReader::new(f);
        loop {
            let mut line = String::with_capacity(100);
            try!(f.read_line(&mut line));
            if line.len() == 0 { break; }
            if line.starts_with("cpu ") {
                let mut pieces = words(&line);
                pieces.next();
                t.add_next_cnt(Key::metric("cpu.user"), &mut pieces);
                t.add_next_cnt(Key::metric("cpu.nice"), &mut pieces);
                t.add_next_cnt(Key::metric("cpu.system"), &mut pieces);
                t.add_next_cnt(Key::metric("cpu.idle"), &mut pieces);
                t.add_next_cnt(Key::metric("cpu.iowait"), &mut pieces);
                t.add_next_cnt(Key::metric("cpu.irq"), &mut pieces);
                t.add_next_cnt(Key::metric("cpu.softirq"), &mut pieces);
                t.add_next_cnt(Key::metric("cpu.steal"), &mut pieces);
                t.add_next_cnt(Key::metric("cpu.guest"), &mut pieces);
                t.add_next_cnt(Key::metric("cpu.guest_nice"), &mut pieces);
            } else if line.starts_with("btime ") {
                boot_time = FromStr::from_str(line[6..].trim()).ok();
            }
        }
        Ok(())
    }).ok();
    File::open(&Path::new("/proc/meminfo")).and_then(|f| {
        let mut f = BufReader::new(f);
        loop {
            let mut line = String::with_capacity(50);
            try!(f.read_line(&mut line));
            if line.len() == 0 { break; }
            let mut pieces = words(&line);
            let ksuffix = if let Some(x) = pieces.next() { x }
                else { continue; };
            let key = Key::metric(&format!("memory.{}",
                                 ksuffix.trim_right_matches(':')));
            let val = match pieces.next() {
                Some(val) => val,
                None => continue,
            };
            let mult = match pieces.next() {
                Some("kB") => 1024,
                Some(x) => {
                    debug!("Unknown memory unit {:?}", x);
                    continue;
                }
                None => 1,
            };
            <i64 as FromStr>::from_str(val)
                .map(|x| t.add(key, Integer(x*mult)))
                .ok();
        }
        Ok(())
    }).ok();
    File::open(&Path::new("/proc/net/dev")).and_then(|f| {
        let mut f = BufReader::new(f);
        let mut line = String::with_capacity(200);
        try!(f.read_line(&mut line));
        let mut line = String::with_capacity(200);
        try!(f.read_line(&mut line));
        let mut slices = line.splitn(3, '|');
        slices.next();
        let mut fields = vec!();
        for i in words(&slices.next().unwrap_or("")) {
            fields.push(format!("net.interface.rx.{}", i));
        }
        for i in words(&slices.next().unwrap_or("")) {
            fields.push(format!("net.interface.tx.{}", i));
        }
        loop {
            let mut line = String::with_capacity(200);
            try!(f.read_line(&mut line));
            if line.len() == 0 { break; }
            let mut pieces = words(&line);
            let interface = pieces.next().unwrap_or("unknown:")
                            .trim_right_matches(':');
            for (k, v) in fields.iter().zip(pieces) {
                FromStr::from_str(v).map(|x|
                    t.add(
                        Key::pairs(&[
                            ("interface", interface),
                            ("metric", &k),
                            ]),
                        Counter(x)))
                .ok();
            }
        }
        Ok(())
    }).ok();
    File::open(&Path::new("/proc/net/netstat")).and_then(|f| {
        let mut f = BufReader::new(f);
        loop {
            let mut header_line = String::with_capacity(2048);
            try!(f.read_line(&mut header_line));
            if header_line.len() == 0 { break; }
            let mut header = words(&header_line);

            let mut values_line = String::with_capacity(1024);
            try!(f.read_line(&mut values_line));
            if values_line.len() == 0 { break; }
            let mut values = words(&values_line);

            let first = header.next();
            if first != values.next() {
                break;
            }
            let prefix = first.unwrap().trim_right_matches(':');
            for (k, v) in header.zip(values) {
                FromStr::from_str(v).map(|x|
                    t.add(
                        Key::metric(&format!("net.{}.{}", prefix, k)),
                        Counter(x)))
                .ok();
            }
        }
        Ok(())
    }).ok();
    File::open(&Path::new("/proc/diskstats")).and_then(|f| {
        let mut f = BufReader::new(f);
        loop {
            let mut line = String::with_capacity(200);
            try!(f.read_line(&mut line));
            if line.len() == 0 { break; }
            let mut pieces = words(&line);
            pieces.next(); pieces.next(); // major, minor numbers
            let device = pieces.next().unwrap_or("loop");
            if device.starts_with("ram") || device.starts_with("loop") {
                // We currently ignore ramdisks and loop devices, because
                // nobody uses ram disks in this decade (there is tmpfs)
                // and because loop devices are rarely used and have
                // diskstats entries even if unused
                continue;
            }
            t.add_next_cnt(Key::pairs(&[
                            ("device", device),
                            ("metric", "disk.read.ops"),
                            ]), &mut pieces);
            t.add_next_cnt(Key::pairs(&[
                            ("device", device),
                            ("metric", "disk.read.merges"),
                            ]), &mut pieces);
            t.add_next_cnt(Key::pairs(&[
                            ("device", device),
                            ("metric", "disk.read.sectors"),
                            ]), &mut pieces);
            t.add_next_cnt(Key::pairs(&[
                            ("device", device),
                            ("metric", "disk.read.time"),
                            ]), &mut pieces);
            t.add_next_cnt(Key::pairs(&[
                            ("device", device),
                            ("metric", "disk.write.ops"),
                            ]), &mut pieces);
            t.add_next_cnt(Key::pairs(&[
                            ("device", device),
                            ("metric", "disk.write.merges"),
                            ]), &mut pieces);
            t.add_next_cnt(Key::pairs(&[
                            ("device", device),
                            ("metric", "disk.write.sectors"),
                            ]), &mut pieces);
            t.add_next_cnt(Key::pairs(&[
                            ("device", device),
                            ("metric", "disk.write.time"),
                            ]), &mut pieces);
            t.add_next_int(Key::pairs(&[
                            ("device", device),
                            ("metric", "disk.in_progress"),
                            ]), &mut pieces);
            t.add_next_cnt(Key::pairs(&[
                            ("device", device),
                            ("metric", "disk.time"),
                            ]), &mut pieces);
            t.add_next_cnt(Key::pairs(&[
                            ("device", device),
                            ("metric", "disk.weighted_time"),
                            ]), &mut pieces);
        }
        Ok(())
    }).ok();
    File::open(&Path::new("/proc/net/tcp")).and_then(|f| {
        #[allow(dead_code, non_camel_case_types)]
        enum S {
            UNKNOWN,
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
        };
        let mut f = BufReader::new(f);
        let mut line = String::with_capacity(200);
        try!(f.read_line(&mut line));
        let mut tx_queue = 0;
        let mut rx_queue = 0;
        let mut cli = HashMap::new();
        let mut serv = HashMap::new();
        let mut close_wait = 0;
        let mut time_wait = 0;
        let mut established = 0;
        let mut listening = 0;
        loop {
            let mut line = String::with_capacity(200);
            try!(f.read_line(&mut line));
            if line.len() == 0 { break; }
            let mut pieces = words(&line);
            pieces.next(); // Skip slot number
            let local = pieces.next()
                .and_then(|x| if x.len() == 13 { Some(x) } else { None })
                .and_then(|x| u16::from_str_radix(&x[9..13], 16).ok())
                .unwrap_or(0u16);
            let remote = pieces.next()
                .and_then(|x| if x.len() == 13 { Some(x) } else { None })
                .and_then(|x| u16::from_str_radix(&x[9..13], 16).ok())
                .unwrap_or(0u16);
            let status = pieces.next()
                .and_then(|x| u16::from_str_radix(x, 16).ok())
                .unwrap_or(0);
            let mut queues = pieces.next().unwrap_or("0:0").split(':');
            let tx = queues.next()
                .and_then(|x| u16::from_str_radix(x, 16).ok())
                .unwrap_or(0);
            let rx = queues.next()
                .and_then(|x| u16::from_str_radix(x, 16).ok())
                .unwrap_or(0);
            {
                // TODO(tailhook) read ephemeral port range
                let mut pair = if local > 0 && local < 32768 {
                    // Consider this inbound connection
                    Some((&mut serv, local))
                } else if remote > 0 && remote < 32768 {
                    // Consider this outbound connection
                    Some((&mut cli, remote))
                } else {
                    // Don't know, probably rare enough to ignore
                    None
                };
                if let Some((ref mut coll, port)) = pair {
                    let estab = if status == S::ESTABLISHED as u16 {1} else {0};
                    match coll.entry(port) {
                        Vacant(e) => {
                            e.insert((estab, tx, rx));
                        }
                        Occupied(mut e) => {
                            let &mut (ref mut pestab, ref mut ptx, ref mut prx)
                                = e.get_mut();
                            *pestab += estab;
                            *ptx += tx;
                            *prx += rx;
                        }
                    };
                }
            }
            tx_queue += tx;
            rx_queue += rx;
            match status {
                x if x == S::ESTABLISHED as u16 => established += 1,
                x if x == S::CLOSE_WAIT as u16 => close_wait += 1,
                x if x == S::TIME_WAIT as u16 => time_wait += 1,
                x if x == S::LISTEN as u16 => listening += 1,
                _ => {}
            }
        }
        t.add(Key::metric("net.tcp.established"), Integer(established));
        t.add(Key::metric("net.tcp.close_wait"), Integer(close_wait));
        t.add(Key::metric("net.tcp.time_wait"), Integer(time_wait));
        t.add(Key::metric("net.tcp.listening"), Integer(listening));
        t.add(Key::metric("net.tcp.tx_queue"), Integer(tx_queue as i64));
        t.add(Key::metric("net.tcp.rx_queue"), Integer(rx_queue as i64));
        for (port, (estab, ptx, prx)) in cli.into_iter() {
            t.add(Key::pairs(&[
                ("metric", "net.tcp.active.established"),
                ("port", &format!("{}", port)),
                ]), Integer(estab));
            t.add(Key::pairs(&[
                ("metric", "net.tcp.active.tx_queue"),
                ("port", &format!("{}", port)),
                ]), Integer(ptx as i64));
            t.add(Key::pairs(&[
                ("metric", "net.tcp.active.rx_queue"),
                ("port", &format!("{}", port)),
                ]), Integer(prx as i64));
        }
        for (port, (estab, ptx, prx)) in serv.into_iter() {
            t.add(Key::pairs(&[
                ("metric", "net.tcp.passive.established"),
                ("port", &format!("{}", port)),
                ]), Integer(estab));
            t.add(Key::pairs(&[
                ("metric", "net.tcp.passive.tx_queue"),
                ("port", &format!("{}", port)),
                ]), Integer(ptx as i64));
            t.add(Key::pairs(&[
                ("metric", "net.tcp.passive.rx_queue"),
                ("port", &format!("{}", port)),
                ]), Integer(prx as i64));
        }
        Ok(())
    }).ok();
    return boot_time;
}
