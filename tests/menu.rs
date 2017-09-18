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

    let menu = nourish_bot::parse_menu(&html);
    assert_eq!(
        menu.entries(),
        vec![
            Entry {
                heading: "Entree Feature".into(),
                items: vec![
                    "Smoked Pork Loin, Corn Bread with Sour Cherries, Rosemary Braised Fennel and Muscat Grape Glaze".into()
                ],
                dietary_info: Some("Contains Pork, Wheat, Dairy, Egg".into()),
            },
            Entry {
                heading: "Meat Pizza".into(),
                items: vec!["Pepperoni".into()],
                dietary_info: Some("Contains Wheat, Dairy, Pork".into())
            },
            Entry {
                heading: "Vegetarian Pizza".into(),
                items: vec!["Margherita".into()],
                dietary_info: Some("Vegetarian - Contains Wheat, Dairy".into())
            },
            Entry {
                heading: "Chef's Special Pizza".into(),
                items: vec!["Chicken Fajita Pizza with Roasted Peppers and Onions, Cheddar, Mozzarella and Cilantro".into()],
                dietary_info: Some("Contains Wheat, Dairy, Spicy".into()),
            },
            Entry {
                heading: "Hearth Oven Sandwich".into(),
                items: vec!["Smoked Turkey Cordon Bleu with Ham, Swiss, Lettuce and Garlic Aioli on Iggys Baguette".into()],
                dietary_info: Some("Contains Wheat, Dairy, Pork, Egg".into()),
            },
            Entry {
                heading: "Panini Special".into(),
                items: vec!["Caprese Grilled Cheese with White Balsamic Marinated Heirloom Tomatoes, Basil, Mozzarella and Pesto".into()],
                dietary_info: Some("Vegetarian - Contains Wheat, Dairy".into()),
            },
            Entry {
                heading: "Meat Soup".into(),
                items: vec!["Chicken and Corn Chowder with Sweet Potatoes and Rosemary".into()],
                dietary_info: Some("Contains Wheat, Dairy".into()),
            },
            Entry {
                heading: "Vegetarian Soup".into(),
                items: vec!["Vegan Carrot and Ginger Bisque".into()],
                dietary_info: Some("Vegan".into()),
            },
        ]
    );
}
