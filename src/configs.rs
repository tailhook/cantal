use std::path::Path;
use std::sync::Arc;

use scan_dir::ScanDir;
use carbon::{Config as Carbon, validator as carbon_validator};
use quire::{parse_config, Options};


#[derive(Clone)]
pub struct Configs {
   pub carbon: Vec<Arc<Carbon>>,
}

pub fn read(dir: &Path) -> Configs {
    let mut configs = Configs {
        carbon: Vec::new(),
    };
    let carbon = carbon_validator();
    let quire = Options::default();
    ScanDir::files().read(dir, |iter| {
        for (entry, name) in iter {
            if name.ends_with(".carbon.yaml") {
                let cfg = match parse_config(entry.path(), &carbon, &quire) {
                    Ok(cfg) => cfg,
                    Err(e) => {
                        warn!("Error reading config {:?}", e);
                        continue;
                    }
                };
                configs.carbon.push(cfg);
            } else {
                warn!("Unknown configuration file {:?}", entry.path());
            }
        }
    }).map_err(|e| warn!("Error reading config dir {:?}: {}", dir, e)).ok();
    return configs;
}
