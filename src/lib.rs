extern crate redis;

use redis::Commands;

#[derive(Debug)]
pub struct Motd {
    pub message: String,
    pub url: Option<String>,
}

pub struct LeagueRedis {
    pub client: redis::Client,
}

impl Motd {
    pub fn parse(s: &str) -> Result<Motd, String> {
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
    pub fn get_motds(&self, lang: &str) -> redis::RedisResult<Vec<Motd>> {
        let motds: Vec<String> = try!(self.client.smembers(motd_key(lang)));
        Ok(motds.iter().filter_map(|s| Motd::parse(&s).ok()).collect())
    }

    // Adds an motd for the given language.
    pub fn add_motd(&self, lang: &str, motd: &Motd) -> redis::RedisResult<()> {
        self.client.sadd(motd_key(lang), motd.to_string())
    }

    // Removes an motd for the given language.
    pub fn remove_motd(&self, lang: &str, motd: &Motd) -> redis::RedisResult<()> {
        self.client.srem(motd_key(lang), motd.to_string())
    }
}
