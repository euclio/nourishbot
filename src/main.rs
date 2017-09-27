extern crate nourish_bot;

#[macro_use]
extern crate clap;

extern crate chrono;
extern crate dotenv;
extern crate reqwest;
extern crate slack_hook;
extern crate webbrowser;

use std::env;

use chrono::{NaiveDate, Local};
use clap::{App, SubCommand, Arg};
use slack_hook::{Slack, PayloadBuilder};

use nourish_bot::errors::*;

fn main() {
    let matches = App::new(crate_name!())
        .version(crate_version!())
        .about(crate_description!())
        .subcommand(SubCommand::with_name("print").about(
            "Print the nourish menu",
        ))
        .subcommand(SubCommand::with_name("open").about(
            "Open the nourish menu in the default web browser",
        ))
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
                .validator(|arg| if NaiveDate::parse_from_str(&arg, "%Y-%m-%d")
                    .is_ok()
                {
                    Ok(())
                } else {
                    Err(String::from("Date is not in YYYY-MM-DD format"))
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

    let markdown = nourish_bot::retrieve_menu(&date).and_then(|menu| menu.to_markdown());

    let markdown = match markdown {
        Ok(markdown) => markdown,
        Err(e) => {
            match e {
                Error(ErrorKind::EmptyMenu, _) => String::from(
                    r"There is no menu today ¯\_(ツ)_/¯",
                ),
                Error(ErrorKind::Network(_), _) => e.to_string(),
                _ => String::from("Unspecified error."),
            }
        }
    };

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
