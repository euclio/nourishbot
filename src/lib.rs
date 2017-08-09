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
use std::fmt::{self, Display, Write};

use chrono::{Datelike, Duration, NaiveDate};
use inflector::Inflector;
use linked_hash_map::LinkedHashMap;
use regex::Regex;
use select::document::Document;
use select::predicate::{Attr, Name};
use url::Url;

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
#[derive(Debug, Clone, Default)]
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

        for item in &self.items {
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
    pub fn entries(&mut self) -> Vec<Entry> {
        self.0.values().cloned().collect()
    }

    /// Renders the menu as a Markdown string.
    pub fn to_markdown(&mut self) -> Option<String> {
        let mut output = String::default();

        for entry in self.entries() {
            writeln!(output, "{}", entry).unwrap();
        }

        if output.is_empty() {
            return None;
        }

        writeln!(
            output,
            "> Made with :cnr: by @anrussell. Source available at \
                  https://github.com/euclio/nourishbot."
        ).unwrap();

        Some(output)
    }
}

/// Returns the URL pointing to the Nourish menu for a given day. On the weekends, returns Friday's
/// menu.
pub fn url_for_date(date: &NaiveDate) -> Url {
    let days_from_monday = cmp::min(date.weekday().num_days_from_monday(), 4);
    let monday = *date - Duration::days(date.weekday().num_days_from_monday() as i64);

    Url::parse(&format!(
        "http://dining.guckenheimer.com/clients/athenahealth/fss/fss.\
                       nsf/weeklyMenuLaunch/8DURSE~{}/$file/day{}.htm",
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

                let mut entry = menu.entry(heading.clone()).or_insert_with(|| {
                    Entry {
                        heading: heading.to_owned(),
                        items: Vec::new(),
                        dietary_info: None,
                    }
                });

                if node.is(Name("div")) {
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
