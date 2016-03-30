extern crate league_motd;
extern crate redis;
extern crate clap;

use std::env;
use std::io::{self, Write};
use std::process::exit;
use clap::{App, SubCommand};
use league_motd::*;

fn get_env(name: &str) -> String {
    match env::var(name) {
        Ok(val) => val,
        Err(_) => {
            println!("{} not set", name);
            exit(1);
        }
    }
}

fn valid_language(lang: &str) -> bool {
    lang == "de" || lang == "en"
}

fn main() {
    let redis_url = get_env("REDIS_URL");
    let league = match redis::Client::open(&*redis_url) {
        Ok(client) => LeagueRedis { client: client },
        Err(err) => {
            println!("Could not connect to Redis: {:?}", err);
            exit(1);
        }
    };

    let app_m = App::new("league-motd")
        .arg_from_usage("-l, --language=<language> 'Language: de/en'")
        .subcommand(SubCommand::with_name("list")
                    .about("Lists all existing motds"))
        .subcommand(SubCommand::with_name("add")
                    .about("Adds a motd")
                    .args_from_usage(
                        "<message>      'the message to show'
                        -u, --url=[url] 'clickable url'"))
        .subcommand(SubCommand::with_name("remove")
                    .about("Interactively removes a motd"))
        .get_matches();

    let lang = app_m.value_of("language").unwrap();
    if !valid_language(lang) {
        println!("Invalid language {}", lang);
        exit(1);
    }

    match app_m.subcommand() {
        ("list",   Some(sub_m)) => cmd_list(lang, &league, &sub_m),
        ("add",    Some(sub_m)) => cmd_add(lang, &league, &sub_m),
        ("remove", Some(sub_m)) => cmd_remove(lang, &league, &sub_m),
        _ => println!("{}", app_m.usage())
    };
}

fn handle_error(err: &redis::RedisError) -> ! {
    println!("Some error occured: {}", err);
    exit(1);
}

fn cmd_list(lang: &str, league: &LeagueRedis, _: &clap::ArgMatches) {
    match league.get_motds(lang) {
        Ok(motds) => {
            for m in motds {
                println!("{:?}", m);
            }
        },
        Err(err) => handle_error(&err)
    }
}

fn cmd_add(lang: &str, league: &LeagueRedis, sub_m: &clap::ArgMatches) {
    let message = sub_m.value_of("message").unwrap().to_string();
    let url = sub_m.value_of("url").map(|s| s.to_string());
    let motd = &Motd { message: message, url: url };
    match league.add_motd(lang, motd) {
        Ok(()) => println!("Success: {:?}", motd),
        Err(err) => handle_error(&err)
    }
}

fn cmd_remove(lang: &str, league: &LeagueRedis, _: &clap::ArgMatches) {
    let motds = league.get_motds(lang).unwrap_or_else(|e| handle_error(&e));
    println!("Choose entry to remove:");
    for (i, m) in motds.iter().enumerate() {
        println!(" {}. {:?}", i + 1, m);
    }
    print!("> "); io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let n: usize = input.trim().parse().ok().expect("Please input a number");
    if n > 0 && n <= motds.len() {
        league.remove_motd(lang, &motds[n - 1]).unwrap_or_else(|e| handle_error(&e));
        println!("Success, removed: {:?}", motds[n - 1]);
    } else {
        println!("Invalid number");
    }
}
