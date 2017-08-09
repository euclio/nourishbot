extern crate nourish_bot;

use std::fs::File;
use std::io::Read;

use nourish_bot::Entry;

#[test]
fn parse_menu() {
    let html = {
        let mut file = File::open("tests/html/nourish-2017-08-09.html").unwrap();
        let mut html = String::new();
        file.read_to_string(&mut html).unwrap();
        html
    };

    let mut menu = nourish_bot::parse_menu(&html);
    assert_eq!(
        menu.entries(),
        vec![
            Entry {
                heading: "Entree Feature".into(),
                items: vec![
                    "Smoked Pork Loin, Corn Bread with Sour Cherries, Rosemary Braised Fennel and Muscat Grape Glaze".into()
                ],
            },
            Entry {
                heading: "Meat Pizza".into(),
                items: vec!["Pepperoni".into()],
            },
            Entry {
                heading: "Vegetarian Pizza".into(),
                items: vec!["Margherita".into()],
            },
            Entry {
                heading: "Chef's Special Pizza".into(),
                items: vec!["Chicken Fajita Pizza with Roasted Peppers and Onions, Cheddar, Mozzarella and Cilantro".into()],
            },
            Entry {
                heading: "Hearth Oven Sandwich".into(),
                items: vec!["Smoked Turkey Cordon Bleu with Ham, Swiss, Lettuce and Garlic Aioli on Iggys Baguette".into()],
            },
            Entry {
                heading: "Panini Special".into(),
                items: vec!["Caprese Grilled Cheese with White Balsamic Marinated Heirloom Tomatoes, Basil, Mozzarella and Pesto".into()],
            },
            Entry {
                heading: "Meat Soup".into(),
                items: vec!["Chicken and Corn Chowder with Sweet Potatoes and Rosemary".into()],
            },
            Entry {
                heading: "Vegetarian Soup".into(),
                items: vec!["Vegan Carrot and Ginger Bisque".into()],
            },
        ]
    );
}
