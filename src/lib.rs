//! Simple crate that parses athenahealth Watertown's Nourish menu, and posts it to a slack channel
//! of choice.

#![warn(missing_docs)]

#[macro_use]
extern crate lazy_static;

extern crate chrono;
extern crate inflector;
extern crate linked_hash_map;
extern crate regex;
extern crate select;
extern crate url;

use std::cmp;
use std::fmt::Write;

use chrono::{Datelike, Duration, NaiveDate};
use inflector::Inflector;
use linked_hash_map::LinkedHashMap;
use regex::Regex;
use select::document::Document;
use select::predicate::{Attr, Name};
use url::Url;

lazy_static! {
    /// This Regex matches nutritional information, to filter it out from the menu output.
    static ref NUTRITION_RE: Regex = Regex::new(r"Cal.*Fat.*Sat.*Sod.*Carbs.*Fib.*Pro").unwrap();

    /// This Regex matches the price information, to filter it out from the menu output.
    static ref PRICE_RE: Regex = Regex::new(r"^\d+\.\d+( ?/ ?(\d+\.\d+|[a-z]+))?$").unwrap();
}

/// The Nourish menu for a given date.
#[derive(Debug, Clone, Default)]
pub struct Menu(LinkedHashMap<String, Vec<String>>);

impl Menu {
    /// Renders the menu as a Markdown string.
    pub fn to_markdown(&self) -> String {
        let mut output = String::default();

        for (ref category, ref items) in &self.0 {
            writeln!(output, "*{}*\n", category).unwrap();

            for item in items.iter() {
                writeln!(output, "• {}", item).unwrap();
            }

            writeln!(output, "").unwrap();
        }

        if output.is_empty() {
            writeln!(output, "There is no menu today ¯\\_(ツ)_/¯").unwrap();
        }

        writeln!(output,
                 "> Made with :cnr: by @anrussell. Source available at \
                  https://github.com/euclio/nourishbot.")
            .unwrap();

        output
    }
}

/// Returns the URL pointing to the Nourish menu for a given day. On the weekends, returns Friday's
/// menu.
pub fn url_for_date(date: &NaiveDate) -> Url {
    let days_from_monday = cmp::min(date.weekday().num_days_from_monday(), 4);
    let monday = *date - Duration::days(date.weekday().num_days_from_monday() as i64);

    Url::parse(&format!("http://dining.guckenheimer.com/clients/athenahealth/fss/fss.\
                       nsf/weeklyMenuLaunch/8DURSE~{}/$file/day{}.htm",
                        monday.format("%m-%d-%Y"),
                        days_from_monday + 1))
        .unwrap()
}

/// Parses the menu information out of HTML.
pub fn parse_menu(html: &str) -> Menu {
    let document = Document::from(html);

    let mut menu: LinkedHashMap<String, Vec<String>> = LinkedHashMap::new();

    let mut last_category = None;

    let menu_node = document.find(Attr("id", "center_text")).next().unwrap();
    for node in menu_node.find(Name("div")) {
        let text = node.text().trim().to_owned();

        if NUTRITION_RE.is_match(&text) || PRICE_RE.is_match(&text) {
            continue;
        }

        if let Some("font-weight:bold;") = node.attr("style") {
            let category = text.to_lowercase().to_title_case();

            // Filter out breakfast.
            if category != "Breakfast Special" {
                last_category = Some(text.to_lowercase().to_title_case());
            }
        } else {
            if let Some(ref category) = last_category {
                if menu.contains_key(category) {
                    let mut entries = menu.get_mut(category).unwrap();
                    entries.push(text);
                } else {
                    menu.insert(category.to_owned(), vec![text]);
                }

            }
        }
    }

    Menu(menu)
}

#[cfg(test)]
mod tests {
    use super::{PRICE_RE, url_for_date};

    use chrono::NaiveDate;

    #[test]
    fn monday() {
        let expected_url = "http://dining.guckenheimer.com/clients/athenahealth/fss/fss.\
                            nsf/weeklyMenuLaunch/8DURSE~04-18-2016/$file/day1.htm";

        assert_eq!(expected_url,
                   &url_for_date(&NaiveDate::from_ymd(2016, 4, 18)).to_string());
    }

    #[test]
    fn saturday() {
        let expected_url = "http://dining.guckenheimer.com/clients/athenahealth/fss/fss.\
                            nsf/weeklyMenuLaunch/8DURSE~04-11-2016/$file/day5.htm";

        assert_eq!(expected_url,
                   &url_for_date(&NaiveDate::from_ymd(2016, 4, 16)).to_string());

    }

    #[test]
    fn price_regex() {
        let patterns = ["7.50", "0.25/oz", "2.15 / 2.65"];

        for pattern in &patterns {
            assert!(PRICE_RE.is_match(pattern), pattern.clone());
        }
    }
}
