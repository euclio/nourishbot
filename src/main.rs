extern crate nourish_bot;

extern crate chrono;
extern crate docopt;
extern crate dotenv;
extern crate reqwest;
extern crate rustc_serialize;
extern crate slack_hook;

use std::env;
use std::io::prelude::*;

use chrono::Local;
use docopt::Docopt;
use slack_hook::{Slack, PayloadBuilder};

const USAGE: &'static str = r"
Nourishbot!

Usage:
    nourishbot help
    nourishbot print
    nourishbot post <slack-channel>...

Subcommands:
    help                   Show this screen.
    print                  Print the Nourish menu.
    post                   Post the menu to the given Slack channels.
";

#[derive(Debug, RustcDecodable)]
struct Args {
    arg_slack_channel: Vec<String>,
}

fn main() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.decode())
        .unwrap_or_else(|e| e.exit());
    dotenv::dotenv().ok();

    let url = nourish_bot::url_for_date(&Local::today().naive_local());

    let menu = {
        let mut res = reqwest::get(&url.to_string()).unwrap();

        let mut bytes = vec![];
        res.read_to_end(&mut bytes).unwrap();
        let body = String::from_utf8_lossy(&bytes);

        nourish_bot::parse_menu(&body)
    };

    println!("{}", menu.to_markdown());

    for channel in &args.arg_slack_channel {
        let slack = Slack::new(env::var("WEBHOOK_URL")
                                   .expect("WEBHOOK_URL is not set")
                                   .as_str())
                .unwrap();
        let p = PayloadBuilder::new()
            .text(menu.to_markdown().as_str())
            .channel(channel.as_str())
            .username("nourishbot")
            .icon_emoji(":athena:")
            .build()
            .unwrap();

        match slack.send(&p) {
            Ok(()) => println!("Posted to {}", channel),
            Err(err) => println!("Error posting to {}: {}", channel, err),
        }
    }
}
