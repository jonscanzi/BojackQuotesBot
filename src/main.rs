use telebot::Bot;
use futures::stream::Stream;
use std::path::Path;
use std::fs::File;
use std::io::{self, BufRead};
use std::env;
use telebot::functions::*;
use rand::prelude::*;
use regex::Regex;
use lazy_static::lazy_static;
use futures::future::Future;

#[derive(Clone, Debug)]
struct Quote {
    quote: String,
    quotee: String,
    season: u8,
    episode: u8,
}

impl Quote {
    pub fn formatted(&self) -> String {
        // Create specific formats for each combination of unknown/known season and ep to make sure format is only called once
        // Note: unknown season/ep means it is 0 in thr quote file
        match self.season {
            0 => {
                match self.episode {
                    0 => format!("\"{}\"\n\n - {}\n Season ?, Episode ?", self.quote, self.quotee),
                    _ => format!("\"{}\"\n\n - {}\n Season ?, Episode {}", self.quote, self.quotee, self.episode),
                }
            },
            _ => {
                match self.episode {
                    0 => format!("\"{}\"\n\n - {}\n Season {}, Episode ?", self.quote, self.quotee, self.season),
                    _ => format!("\"{}\"\n\n - {}\n Season {}, Episode {}", self.quote, self.quotee, self.season, self.episode),
                }
            },
        }
    }
}
    
lazy_static! {
    static ref ALL_QUOTES: Vec<Quote> = parse_psv("quotes.psv");
}

#[inline]
fn get_random_quote() -> String {
    let rand_num: usize = thread_rng().gen_range(0, ALL_QUOTES.len());
    ALL_QUOTES[rand_num].formatted()
}

#[inline]
fn get_random_quote_from_season(season: u8) -> String {
    let specific_season_quotes: Vec<&Quote> = ALL_QUOTES.iter().filter(|q| q.season == season).collect();
    if specific_season_quotes.is_empty() {
        return "No quotes found for that season. Life sucks, I know.".to_string();
    }
    let rand_num: usize = thread_rng().gen_range(0, specific_season_quotes.len());
    specific_season_quotes[rand_num].formatted()
}

/// Basically just recovering the quotes from the file and then running the bot
#[tokio::main]
async fn main() {
    // Create the bot
    let mut bot = Bot::new(&env::var("TELEGRAM_BOJACKQUOTESBOT_TOKEN").expect("Error: could not load token environment variable")).update_interval(300);
    
    // Register a reply command which answers a message
    let quote_handle = bot.new_cmd("/quote")
        .and_then(|(bot, msg)| {
            let text; //TODO: change this to a block
            if msg.text.unwrap().contains("plz") {
                text = "plz kill me ;____________;".to_string();
            }
            else {
                text = get_random_quote();
            }
            
            bot.message(msg.chat.id, text).send()
        })
        .for_each(|_| Ok(()));

    let season_handle = bot.new_cmd("/season")
        .and_then(|(bot, msg)| {
            let season_re = Regex::new(r"(\d+)").unwrap(); //TODO: make regex static to avoid repeated compilations
            let text: String = {

                // let season: Option<u8> = msg.text.and_then(|message| {
                //     season_re.captures(&message).and_then(|caps| caps.get(1)).and_then(|cap1| {cap1.as_str().parse().ok()})
                // });

                let season: Option<u8> = msg.text.and_then(|message| {
                    season_re.captures(&message).and_then(|caps| caps.get(1)).and_then(|cap1| {cap1.as_str().parse().ok()})
                });

                match season {
                    Some(season) => get_random_quote_from_season(season),
                    None => "I didn't unserstand the season you selected. But then again, I usually don't understand much of anything.".to_string(),
                }
            };
            bot.message(msg.chat.id, text).send()
        })
        .for_each(|_| Ok(()));
    bot.run_with(quote_handle.join(season_handle));
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