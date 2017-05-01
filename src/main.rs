extern crate nourish_bot;

#[macro_use]
extern crate clap;

extern crate chrono;
extern crate dotenv;
extern crate reqwest;
extern crate slack_hook;

use std::env;
use std::io::prelude::*;

use chrono::Local;
use clap::{App, SubCommand, Arg};
use slack_hook::{Slack, PayloadBuilder};

fn main() {
    let matches = App::new(crate_name!())
        .version(crate_version!())
        .about(crate_description!())
        .subcommand(SubCommand::with_name("print").about("Print the nourish menu"))
        .subcommand(SubCommand::with_name("post")
                        .about("Post the nourish menu to the given Slack channels")
                        .arg(Arg::with_name("slack-channel")
                                 .required(true)
                                 .multiple(true)
                                 .help("A Slack channel (#food) or username (@anrussell)")))
        .get_matches();

    dotenv::dotenv().ok();

    let url = nourish_bot::url_for_date(&Local::today().naive_local());

    let menu = {
        let mut res = reqwest::get(&url.to_string()).unwrap();

        let mut bytes = vec![];
        res.read_to_end(&mut bytes).unwrap();
        let body = String::from_utf8_lossy(&bytes);

        nourish_bot::parse_menu(&body)
    };

    let markdown = menu.to_markdown()
        .unwrap_or_else(|| r"There is no menu today ¯\_(ツ)_/¯".to_string());

    println!("{}", markdown);

    if let Some("print") = matches.subcommand_name() {
        return;
    }

    let channels = matches
        .values_of("slack-channel")
        .map(Iterator::collect)
        .unwrap_or_else(|| vec![]);

    for channel in &channels {
        let slack = Slack::new(env::var("WEBHOOK_URL")
                                   .expect("WEBHOOK_URL is not set")
                                   .as_str())
                .unwrap();
        let p = PayloadBuilder::new()
            .text(markdown.as_str())
            .channel(*channel)
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
