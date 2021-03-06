extern crate nourish_bot;

#[macro_use]
extern crate clap;

extern crate chrono;
extern crate dotenv;
extern crate reqwest;
extern crate slack_hook;
extern crate webbrowser;

use std::env;
use std::fmt::Write;

use chrono::{Local, NaiveDate};
use clap::{App, Arg, SubCommand};
use slack_hook::{PayloadBuilder, Slack};

use nourish_bot::errors::*;

static FOOTER: &str = "> Made with :btb: by @anrussell. Please direct feature requests and bug \
                       reports to https://github.com/euclio/nourishbot";

fn main() {
    let matches = App::new(crate_name!())
        .version(crate_version!())
        .about(crate_description!())
        .subcommand(SubCommand::with_name("print").about("Print the nourish menu"))
        .subcommand(
            SubCommand::with_name("open").about("Open the nourish menu in the default web browser"),
        )
        .subcommand(
            SubCommand::with_name("post")
                .about("Post the nourish menu to the given Slack channels")
                .arg(
                    Arg::with_name("slack-channel")
                        .required(true)
                        .multiple(true)
                        .help("A Slack channel (#food) or username (@anrussell)"),
                ),
        )
        .arg(
            Arg::with_name("date")
                .long("date")
                .short("d")
                .takes_value(true)
                .help("the date that should be used to pull the menu")
                .validator(|arg| {
                    if NaiveDate::parse_from_str(&arg, "%Y-%m-%d").is_ok() {
                        Ok(())
                    } else {
                        Err(String::from("Date is not in YYYY-MM-DD format"))
                    }
                }),
        )
        .get_matches();

    dotenv::dotenv().ok();

    let date = matches
        .value_of("date")
        .map(|arg| NaiveDate::parse_from_str(&arg, "%Y-%m-%d").unwrap())
        .unwrap_or_else(|| Local::today().naive_local());

    let url = nourish_bot::url_for_date(&date);

    if let Some("open") = matches.subcommand_name() {
        webbrowser::open(&url.to_string()).expect("problem opening web browser");
        return;
    }

    let menu = nourish_bot::retrieve_menu(&date).and_then(|menu| menu.to_markdown());

    let message = match menu {
        Ok(menu) => menu,
        Err(e) => match e {
            Error(ErrorKind::EmptyMenu, _) => {
                String::from(r"The menu was empty today. Is it a holiday? ¯\_(ツ)_/¯")
            }
            Error(ErrorKind::Network(_), _) => {
                let mut message = String::new();
                writeln!(
                    message,
                    "Sorry, there was a problem retrieving the menu. \
                     I tried this link: {}",
                    url,
                ).unwrap();
                writeln!(
                    message,
                    "This usually means that the menu for this week hasn't been posted yet. \
                     Try the link again later.",
                ).unwrap();
                writeln!(message).unwrap();
                writeln!(message, "_Error:_ `{}`", e.to_string()).unwrap();
                message
            }
            _ => String::from("Something went wrong while trying to get the menu. This is a bug."),
        },
    };

    let markdown = format!("{}{}", message, FOOTER);

    println!("{}", markdown);

    if let Some("print") = matches.subcommand_name() {
        return;
    }

    if let Some(sub_matches) = matches.subcommand_matches("post") {
        let channels = sub_matches
            .values_of("slack-channel")
            .map(Iterator::collect)
            .unwrap_or_else(|| vec![]);

        for channel in &channels {
            let slack = Slack::new(
                env::var("WEBHOOK_URL")
                    .expect("WEBHOOK_URL is not set")
                    .as_str(),
            ).unwrap();
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
}
