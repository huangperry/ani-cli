#![allow(unused)]
use std::io::{stdin, stdout, Write};
use std::str;
use std::vec;
use structopt::StructOpt;
use curl::easy::Easy;
use regex::Regex;
use open;

const BASE_URL: &str = "https://www2.gogoanime.cm";

/// Search for a pattern in a file and display the lines that contain it.
#[derive(StructOpt)]
struct Cli {
    /// Activate debug mode
    #[structopt(short = "d")]
    download: bool,

    /// Continue where you left off
    #[structopt(short = "H")]
    cont: bool,

    /// Delete history
    #[structopt(short = "D")]
    delete: bool,

    /// Set video quality (**best**/worst/360/480/720/1080)
    #[structopt(short = "q", default_value = "best")]
    quality: String,

    /// Play the dub version
    #[structopt(long = "dub")]
    dub: bool,

    /// Play in VLC
    #[structopt(short = "v")]
    vlc: bool,

    query: String,
}

fn search_anime(query: String) -> Vec<String> {
    /// Get anime name and id
    let search = query.replace(' ', "-");
    let search_url = [BASE_URL, "//search.html?keyword=", search.as_str()].concat();
    let mut handle = Easy::new();
    handle.url(search_url.as_str()).unwrap();
    let mut list = vec![];
    {
        let mut transfer = handle.transfer();
        transfer.write_function(|data| {
            let buffer = match str::from_utf8(&data) {
                Ok(v) => v,
                Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
            };
            let re = Regex::new(r#"[[:space:]]*<a href="/category/([^"]*)" title="([^"]*)">.+?"#).unwrap();
            for cap in re.captures_iter(buffer) {
                list.push(cap.get(1).map_or("", |m| m.as_str()).to_string());
            }
            Ok(data.len())
        }).unwrap();
        transfer.perform().unwrap();
    }
    list
}

fn search_eps(anime_id: &String) -> (u32,u32) {
    // get available episodes for anime_id
    let search_url = [BASE_URL, "/category/", anime_id.as_str()].concat();
    let mut handle = Easy::new();
    handle.url(search_url.as_str()).unwrap();
    let mut ep_start: u32 = 0;
    let mut ep_end: u32 = 0;
    {
        let mut transfer = handle.transfer();
        transfer.write_function(|data| {
            let buffer = match str::from_utf8(data) {
                Ok(v) => v,
                Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
            };
            let re = Regex::new(r#"class="active" ep_start = '([0-9]*)' ep_end = '([0-9]*)'"#).unwrap();
            for cap in re.captures_iter(buffer) {
                ep_start = cap.get(1).map_or("", |m| m.as_str()).parse::<u32>().unwrap();
                ep_end = cap.get(2).map_or("", |m| m.as_str()).parse::<u32>().unwrap();
            }
            Ok(data.len())
        }).unwrap();
        transfer.perform().unwrap();
    }
    (ep_start + 1, ep_end)
}

fn select_anime(list: &Vec<String>) -> u32 {
    print!("Enter number: ");
    stdout().flush();
    let mut s = String::new();
    stdin().read_line(&mut s).expect("Did not enter a correct string");
    let index = s.trim_end().parse::<u32>().unwrap();
    println!("You selected: {}", list[(index - 1) as usize]);
    index - 1
}

fn display_search(list: &Vec<String>) {
    for (i, anime) in list.iter().enumerate() {
        println!("[{}] {}", i + 1, anime);
    }
}

fn select_ep(start: u32, end: u32) -> u32{
    print!("Choose episode [{}-{}]: ", start, end);
    stdout().flush();
    let mut s = String::new();
    stdin().read_line(&mut s).expect("Did not enter a correct string");
    s.trim_end().parse::<u32>().unwrap()
}

fn get_links(anime: &String, ep: u32) -> (String, String) {
    let search_url = [BASE_URL, "/", anime.as_str(), "-episode-", ep.to_string().as_str()].concat();
    let mut handle = Easy::new();
    handle.url(search_url.as_str()).unwrap();
    let mut embedded_link = String::new();
    {
        let mut transfer = handle.transfer();
        transfer.write_function(|data| {
            let buffer = match str::from_utf8(data) {
                Ok(v) => v,
                Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
            };
            let re = Regex::new(r#"rel="100" data-video="([^"]*)".*"#).unwrap();
            for cap in re.captures_iter(buffer) {
                embedded_link = String::from(cap.get(1).map_or("", |m| m.as_str()));
            }
            Ok(data.len())
        }).unwrap();
        transfer.perform().unwrap();
    }

    let embedded_video_url = embedded_link.clone();
    let re = Regex::new(r#"id.+?&"#).unwrap();
    let episode_id = re.find(embedded_video_url.as_str()).unwrap().as_str();
    let video_url= ["https://gogoplay1.com/download?".to_string(), episode_id.to_string()].concat();
    (embedded_video_url, video_url)
}

fn get_video_quality() {

}

fn open_episode(anime_id: String, ep: u32) {
    println!("Getting data for episode {}", ep);

}

fn main() {
    let args = Cli::from_args();
    let mut search_results = search_anime(args.query);
    display_search(&search_results);
    let index = select_anime(&search_results);
    let anime = &search_results[index as usize];
    let (start_ep, end_ep) = search_eps(anime);
    let ep = select_ep(start_ep, end_ep);
    let (embed_url, video_url) = get_links(anime, ep);
    open::that(["https:", embed_url.as_str()].concat()).unwrap();
}
