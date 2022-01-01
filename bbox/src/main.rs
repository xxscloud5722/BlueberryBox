use std::collections::HashMap;
use std::env;

mod server;
mod core;
mod setting;
mod global;


/// parse args to hashMap
fn parse_args(args: Vec<String>) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for i in 2..args.len() {
        if i % 2 != 0 {
            continue;
        }
        map.insert(String::from(&args.get(i).unwrap().trim()[1..]),
                   String::from(args.get(i + 1).unwrap().trim()));
    }
    map
}

/// print module
pub mod print_box {
    pub fn print_version() {
        println!("Welcome to BlueberryBox (Rust) Service !");
        println!("Version: 2.2 20220108");
        println!("GitHub: https://github.com/xxscloud5722/BlueberryBox");
    }

    pub fn print_banner() {
        print_version();
        println!(r##"
______  _               _                              ______
| ___ \| |             | |                             | ___ \
| |_/ /| | _   _   ___ | |__    ___  _ __  _ __  _   _ | |_/ /  ___  __  __
| ___ \| || | | | / _ \| '_ \  / _ \| '__|| '__|| | | || ___ \ / _ \ \ \/ /
| |_/ /| || |_| ||  __/| |_) ||  __/| |   | |   | |_| || |_/ /| (_) | >  <
\____/ |_| \__,_| \___||_.__/  \___||_|   |_|    \__, |\____/  \___/ /_/\_\
                                                  __/ |
                                                 |___/
    "##);
    }

    pub fn print_info() {
        print_version();
        println!(r##"
v or version            print product version and exit
t or template           output config template
s or server [-options]
  -p    3000            server port, default 3000
  -c    ./config.json   config read path, default current path
  -path ./              scan static file path, defalut current path
  -log  ./log           output log path, defalut current path"##);
    }
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    // No Args
    if args.len() <= 1 {
        print_box::print_info();
        return;
    }

    // version
    if args[1] == "v" || args[1] == "version" {
        print_box::print_version();
        return;
    }

    // config
    if args[1] == "t" || args[1] == "template" {
        setting::output_config_json().await.unwrap();
        return;
    }

    // server
    if args[1] == "s" || args[1] == "server" {
        print_box::print_banner();
        server::start(parse_args(args)).await.unwrap();
        return;
    }
}


