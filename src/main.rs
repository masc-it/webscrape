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

    Ok(())
}

