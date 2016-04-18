extern crate chrono;
extern crate docopt;
extern crate hyper;
extern crate nourish_bot;
extern crate rustc_serialize;
extern crate slack_hook;

use std::io::prelude::*;

use chrono::Local;
use docopt::Docopt;
use hyper::Client;
use hyper::header::Connection;
use slack_hook::{Slack, Payload, PayloadTemplate};

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
    let args: Args = Docopt::new(USAGE).and_then(|d| d.decode()).unwrap_or_else(|e| e.exit());
    let url = nourish_bot::url_for_date(&Local::today().naive_local());

    let menu = {
        let client = Client::new();

        let mut res = client.get(url)
                            .header(Connection::close())
                            .send()
                            .unwrap();

        let mut body = String::new();
        res.read_to_string(&mut body).unwrap();

        nourish_bot::parse_menu(&body)
    };

    println!("{}", menu.to_markdown());

    for channel in &args.arg_slack_channel {
        // TODO: Post to Slack
        let slack = Slack::new("");
        let p = Payload::new(PayloadTemplate::Complete {
            text: Some(&menu.to_markdown()),
            channel: Some(channel),
            username: Some("nourishbot"),
            icon_url: None,
            icon_emoji: Some(":athena:"),
            attachments: None,
            unfurl_links: Some(true),
            link_names: Some(false),
        });

        match slack.send(&p) {
            Ok(()) => println!("Posted to {}", channel),
            Err(err) => println!("Error posting to {}: {}", channel, err),
        }
    }
}
