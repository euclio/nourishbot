//! Simple crate that parses athenahealth Watertown's Nourish menu, and posts it to a slack channel
//! of choice.

#![warn(missing_docs)]

#[macro_use]
extern crate error_chain;

#[macro_use]
extern crate lazy_static;

extern crate chrono;
extern crate inflector;
extern crate linked_hash_map;
extern crate regex;
extern crate reqwest;
extern crate select;

use std::cmp;
use std::fmt::{self, Display, Write};
use std::io::prelude::*;

use chrono::{Datelike, Duration, NaiveDate};
use inflector::Inflector;
use linked_hash_map::LinkedHashMap;
use regex::Regex;
use reqwest::Url;
use select::document::Document;
use select::predicate::{Attr, Name};

pub mod errors;

use errors::*;

lazy_static! {
    /// Extracts the ingredients out of the text of a menu item.
    static ref INGREDIENTS_RE: Regex =
        Regex::new(r"Ingredients: (.*?)(?: - Serving Size.*)?\s*$").unwrap();

    /// This Regex matches nutritional information, to filter it out from the menu output.
    static ref NUTRITION_RE: Regex = Regex::new(r"Cal.*Fat.*Sat.*Sod.*Carbs.*Fib.*Pro").unwrap();

    /// This Regex matches the price information, to filter it out from the menu output.
    static ref PRICE_RE: Regex = Regex::new(r"^\d+\.\d+( ?/ ?(\d+\.\d+|[a-z]+))?$").unwrap();
}

/// The Nourish menu for a given date.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Menu(LinkedHashMap<String, Entry>);

/// A section of the menu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    /// The section header.
    pub heading: String,

    /// The food items listed under the header.
    pub items: Vec<String>,

    /// Information indicating what ingredients are in the food, including whether it's vegetarian,
    /// vegan, etc.
    pub dietary_info: Option<String>,
}

impl Display for Entry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "*{}*\n", self.heading)?;

        let items = if self.items.is_empty() {
            vec![String::from("Nothing today")]
        } else {
            self.items.clone()
        };

        for item in &items {
            let dietary_info = if let Some(ref info) = self.dietary_info {
                format!(" (_{}_)", info)
            } else {
                String::default()
            };

            writeln!(f, "• {}{}", item, dietary_info)?;
        }

        Ok(())
    }
}

impl Menu {
    /// Returns the entries of the menu.
    pub fn entries(&self) -> Vec<Entry> {
        self.0.values().cloned().collect()
    }

    /// Renders the menu as a Markdown string.
    pub fn to_markdown(&self) -> Result<String> {
        let mut output = String::default();

        for entry in self.entries() {
            write!(output, "{}", entry).unwrap();
        }

        if output.is_empty() {
            bail!(ErrorKind::EmptyMenu);
        }

        Ok(output)
    }
}

/// Retrieves and parses the menu for a given date.
pub fn retrieve_menu(date: &NaiveDate) -> Result<Menu> {
    let url = url_for_date(date);

    let mut res = reqwest::get(url.as_str()).map_err(|e| {
        ErrorKind::Network(e.to_string())
    })?;

    let body = if res.status().is_success() {
        let mut bytes = vec![];
        res.read_to_end(&mut bytes)?;
        String::from_utf8_lossy(&bytes).into_owned()
    } else {
        bail!(ErrorKind::Network(res.status().to_string()))
    };

    Ok(parse_menu(&body))
}


/// Returns the URL pointing to the Nourish menu for a given day. On the weekends, returns Friday's
/// menu.
pub fn url_for_date(date: &NaiveDate) -> Url {
    let days_from_monday = cmp::min(date.weekday().num_days_from_monday(), 4);
    let monday = *date - Duration::days(date.weekday().num_days_from_monday() as i64);

    Url::parse(&format!(
        "http://dining.guckenheimer.com/clients/athenahealth/fss/fss.nsf\
        /weeklyMenuLaunch/8DURSE~{}/$file/day{}.htm",
        monday.format("%m-%d-%Y"),
        days_from_monday + 1
    )).unwrap()
}

/// Parses the menu information out of HTML.
pub fn parse_menu(html: &str) -> Menu {
    let document = Document::from(html);
    let menu_node = document.find(Attr("id", "center_text")).next().unwrap();

    let mut menu = LinkedHashMap::new();
    let mut current_heading = None;

    for node in menu_node.children() {
        let text = node.text().trim().to_owned();

        if let Some("font-weight:bold;") = node.attr("style") {
            current_heading = Some(text.to_lowercase().to_title_case());
        } else {
            if let Some(ref heading) = current_heading {
                // Skip breakfast
                if heading == "Breakfast Special" {
                    continue;
                }

                // Skip nutrition and price information.
                if NUTRITION_RE.is_match(&text) || PRICE_RE.is_match(&text) {
                    continue;
                }

                let entry = menu.entry(heading.clone()).or_insert_with(|| {
                    Entry {
                        heading: heading.to_owned(),
                        items: Vec::new(),
                        dietary_info: None,
                    }
                });

                if node.is(Name("div")) && !text.is_empty() {
                    entry.items.push(text);
                } else if let Some(caps) = INGREDIENTS_RE.captures(&text) {
                    entry.dietary_info = Some(caps[1].to_owned());
                }
            } else {
                println!("encountered entry without a heading");
                continue;
            }
        }
    }

    Menu(menu)
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use linked_hash_map::LinkedHashMap;

    use super::{Entry, Menu};

    #[test]
    fn parse_menu() {
        let html = r#"
            <html>
            <body>
            <table>
                <tr>
                    <td id="center_text">
                        <div style="font-weight:bold;">CHEF'S SPECIAL PIZZA</div>
                        <div> </div>
                        <br>
                    </td>
                </tr>
            </table>
            </body>
            </html>
        "#;

        let menu = super::parse_menu(html);
        let expected = {
            let mut menu = LinkedHashMap::new();
            menu.insert(
                String::from("Chef's Special Pizza"),
                Entry {
                    heading: String::from("Chef's Special Pizza"),
                    items: vec![],
                    dietary_info: None,
                },
            );
            Menu(menu)
        };

        assert_eq!(menu, expected);
    }

    #[test]
    fn monday() {
        let expected_url = "http://dining.guckenheimer.com/clients/athenahealth/fss/fss.\
                            nsf/weeklyMenuLaunch/8DURSE~04-18-2016/$file/day1.htm";

        assert_eq!(
            expected_url,
            &super::url_for_date(&NaiveDate::from_ymd(2016, 4, 18)).to_string()
        );
    }

    #[test]
    fn saturday() {
        let expected_url = "http://dining.guckenheimer.com/clients/athenahealth/fss/fss.\
                            nsf/weeklyMenuLaunch/8DURSE~04-11-2016/$file/day5.htm";

        assert_eq!(
            expected_url,
            &super::url_for_date(&NaiveDate::from_ymd(2016, 4, 16)).to_string()
        );

    }

    #[test]
    fn ingredients_regex() {
        use super::INGREDIENTS_RE;

        let test_cases = [
            ("Ingredients: Pork, Egg, Spicy", "Pork, Egg, Spicy"),
            (
                "Ingredients: Vegetarian - Contains Wheat, Dairy",
                "Vegetarian - Contains Wheat, Dairy",
            ),
            ("Ingredients: Vegan - Serving Size 12oz", "Vegan"),
        ];

        for &(text, ingredients) in &test_cases {
            let capture = INGREDIENTS_RE.captures(text).unwrap().get(1).unwrap();
            assert_eq!(capture.as_str(), ingredients);
        }
    }

    #[test]
    fn price_regex() {
        use super::PRICE_RE;

        let patterns = ["7.50", "0.25/oz", "2.15 / 2.65"];

        for pattern in &patterns {
            assert!(PRICE_RE.is_match(pattern), pattern.clone());
        }
    }
}
