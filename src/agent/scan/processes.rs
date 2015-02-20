use std::str::FromStr;
use std::io::BufferedReader;
use std::io::EndOfFile;
use std::io::fs::{File, readdir};
use std::collections::BTreeMap;



struct Group {
    pids: Vec<u32>,
    path: Path,
}

struct Processes {
    groups: BTreeMap<String, Group>,
}

fn get_env_var(pid: u32) -> Option<Path> {
    File::open(&Path::new(format!("/proc/{}/environ", pid)))
    .map(|f| BufferedReader::new(f))
    .and_then(|mut f| {
        loop {
            let line = match f.read_until(0) {
                Ok(line) => line,
                Err(ref e) if e.kind == EndOfFile => {
                    return Ok(None);
                }
                Err(e) => return Err(e),
            };

            if line.starts_with(b"CANTAL_PATH=") {
                return Ok(Some(Path::new(
                    &line["CANTAL_PATH=".len()..line.len()-1])));
            }
        }
    })
    .map_err(|e| debug!("Can't read environ file: {}", e))
    .ok()
    .and_then(|opt| opt)
}

fn scan_groups(pids: Vec<u32>) -> Result<BTreeMap<String, Group>, ()> {
    let groups = BTreeMap::new();
    for pid in pids.into_iter() {
        let cantal_path = get_env_var(pid);
        if cantal_path.is_some() {
            println!("Path {:?} {:?}", pid, cantal_path);
        }
    }
    return Ok(groups);
}

pub fn read() -> Processes {
    return Processes {
        groups: readdir(&Path::new("/proc"))
            .map_err(|e| error!("Error listing /proc: {}", e))
            .map(|lst| lst.into_iter()
                            .map(|x| x.filename_str()
                                      .and_then(FromStr::from_str))
                            .filter(|x| x.is_some())
                            .map(|x| x.unwrap())
                            .collect::<Vec<u32>>())
            .and_then(|lst| scan_groups(lst))
            .unwrap_or(BTreeMap::new()),
    };
}
