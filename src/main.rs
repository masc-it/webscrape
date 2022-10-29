mod webscrape;

use headless_chrome::{Browser, LaunchOptions, Element};
use scraper::{Html, Selector};
use webscrape::{pipeline::ActionData, scraper::Scraper};

use crate::webscrape::pipeline::Actionable;

fn main() -> Result<(), Box<dyn std::error::Error>> {


    let mut scraper = Scraper::default();

    scraper.navigate_to(&String::from("https://httpbin.org/ip"));

    let elements = scraper.find_elements_by_css(&String::from("body"));

    for el in elements {
        println!("{:?}", el.get_inner_text().unwrap());
    }

    println!("XPATH..");
    let elements = scraper.find_elements_by_xpath(&String::from("//body"));

    for el in elements {
        println!("{:?}", el.get_inner_text().unwrap());
    }

    println!("--------------------");
    scraper.navigate_to(&String::from("https://stackoverflow.com/questions/58787864/changing-primary-palette-color-when-using-kivymd-has-no-effect-on-buttons"));

    println!("XPATH..");
    let elements = scraper.find_elements_by_xpath(&String::from("//a[contains(@href, 'lastact')]"));

    for el in elements {
        println!("{:?}", el.get_inner_text().unwrap());
    }


    Ok(())
}

