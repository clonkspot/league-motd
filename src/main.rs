extern crate league_motd;
extern crate redis;

use std::env;
use league_motd::*;

fn main() {
    let redis_url = match env::var("REDIS_URL") {
        Ok(val) => val,
        Err(_) => {
            println!("REDIS_URL not set");
            std::process::exit(1);
        }
    };
    let client = redis::Client::open(&*redis_url).unwrap();
    let league = LeagueRedis { client: client };

    league.add_motd("de", &Motd { message: "Hallo Welt".to_string(), url: Some("http://example.com".to_string()) }).unwrap();
    println!("de motds: {:?}", league.get_motds("de").unwrap());
}
