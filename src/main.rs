extern crate redis;

use std::env;
use redis::Commands;

#[derive(Debug)]
struct Motd {
    message: String,
    url: Option<String>,
}

struct LeagueRedis {
    client: redis::Client,
}

impl Motd {
    fn parse(s: &str) -> Result<Motd, String> {
        let parts: Vec<&str> = s.split("\n").collect();
        match parts.len() {
            1 => Ok(Motd { message: parts[0].to_string(), url: None }),
            2 => {
                if parts[1].starts_with("MOTDURL=") {
                    Ok(Motd { message: parts[0].to_string(), url: Some(parts[1][8..].to_string()) })
                } else {
                    Err(s.to_string())
                }
            }
            _ => Err(s.to_string())
        }
    }
}

impl ToString for Motd {
    fn to_string(&self) -> String {
        match self.url {
            Some(ref url) => format!("{}\nMOTDURL={}", self.message, url),
            None =>          self.message.clone()
        }
    }
}

fn motd_key(lang: &str) -> String {
    format!("league:motd:{}", lang)
}

impl LeagueRedis {
    // Returns all valid motds of the given language.
    fn get_motds(&self, lang: &str) -> redis::RedisResult<Vec<Motd>> {
        let motds: Vec<String> = try!(self.client.smembers(motd_key(lang)));
        Ok(motds.iter().filter_map(|s| Motd::parse(&s).ok()).collect())
    }

    // Adds an motd for the given language.
    fn add_motd(&self, lang: &str, motd: &Motd) -> redis::RedisResult<()> {
        self.client.sadd(motd_key(lang), motd.to_string())
    }

    // Removes an motd for the given language.
    fn remove_motd(&self, lang: &str, motd: &Motd) -> redis::RedisResult<()> {
        self.client.srem(motd_key(lang), motd.to_string())
    }
}

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
