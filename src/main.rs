use telebot::Bot;
use futures::stream::Stream;
use std::path::Path;
use std::fs::File;
use std::io::{self, BufRead};
use std::env;
use telebot::functions::*;
use rand::prelude::*;
use regex::Regex;

#[derive(Clone, Debug)]
struct Quote {
    quote: String,
    quotee: String,
    season: u8,
    episode: u8,
}

impl Quote {
    pub fn formatted(&self) -> String {
        format!("\"{}\"\n\n - {}\n Season {}, Episode {}", self.quote, self.quotee, self.season, self.episode)
    }
}

/// Basically just recovering the quotes from the file and then running the bot
fn main() {
    
    let all_quotes = parse_psv("quotes.psv");
    // Create the bot
    let mut bot = Bot::new(&env::var("TELEGRAM_BOJACKQUOTESBOT_TOKEN").expect("Error: could not load token environment variable")).update_interval(300);
    
    // Register a reply command which answers a message
    let handle = bot.new_cmd("/quote")
        .and_then(move |(bot, msg)| {
            let rand_num: usize = thread_rng().gen_range(0, all_quotes.len());
            bot.message(msg.chat.id, all_quotes[rand_num].formatted()).send()
        })
        .for_each(|_| Ok(()));
    bot.run_with(handle);
}

/// Parse the specific bojack quotes psv (pipe-separated values) file
fn parse_psv<P: AsRef<Path>>(path: P) -> Vec<Quote> {

    let mut ret: Vec<Quote> = Vec::new();
    if let Ok(lines) = read_lines(path) {
        // Consumes the iterator, returns an (Optional) String
        for (idx, line) in lines.enumerate() {
            if let Ok(ip) = line {
                let test: Vec<String> = ip.split("|").map(|t| t.to_string()).collect();

                let quote = test.get(0).expect(&format!("Error: could not read quote at line {}", idx+1)).trim();
                let author = test.get(1).expect(&format!("Error: could not read author name at line {}", idx+1)).trim();
                let season_ep = test.get(2).expect(&format!("Error: could not read season and episode at line {}", idx+1)).trim();
                let season_ep_re = Regex::new(r"[sS](\d+)[eE](\d+)").unwrap(); //TODO: make regex static to avoid repeated compilations

                let caps = season_ep_re.captures(season_ep).unwrap();
                let season: u8 = caps.get(1).unwrap().as_str().parse().unwrap();
                let episode: u8 = caps.get(2).unwrap().as_str().parse().unwrap();

                let new_elem = Quote {
                    quote: quote.to_string(),
                    quotee: author.to_string(),
                    season: season,
                    episode: episode,
                };
                ret.push(new_elem);
            }
        }
    }
    ret
}

/// From rust docs
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}