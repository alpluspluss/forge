mod conf;

use std::path::PathBuf;
use structopt::StructOpt;
use crate::conf::Config;

#[derive(Debug, StructOpt)]
#[structopt(name = "forge", about "A modern C & C++ build system,")]
enum Forge {
    #[structopt(name = "build")]
    Build {
        #[structopt(parse(from_os_str))]
        path: Option<PathBuf>
    }
}

fn main() {
    let opt = Forge::from_args();
    match opt {
        Forge::Build { path } => {
            let path = path.unwrap_or_else(|| std::env::current_dir().unwrap());
                println!("Building project at: {}", path.display());

            let conf_path = path.join("forge.toml");
            match Config::load(&conf_path) {
                Ok(config) => println!("Loaded config: {:?}", config),
                Err(e) => eprintln!("Failed to load config: {}", e)
            }
        }
    }
}