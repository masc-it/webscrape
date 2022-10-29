mod webscrape;

use crate::webscrape::{scraper::{ScraperBuilder, Builder}, proxy::SimpleProxy};


fn main() -> Result<(), Box<dyn std::error::Error>> {


    let proxies = SimpleProxy::from_csv(
        "".to_string(), 
        ":".to_string(), 
        0, 
        1, 
        2, 
        3
    );

    let mut builder = ScraperBuilder::default();

    let mut scraper = builder
        .set_default_timeout(5)
        .set_proxies(proxies)
        .build();

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
    scraper.navigate_to(&String::from("https://whatismycountry.com/"));

    println!("XPATH..");
    let elements = scraper.find_elements_by_xpath(&String::from("//*[@id=\"country\"]"));

    println!("{}", elements.len());
    for el in elements {
        println!("{:?}", el.get_inner_text().unwrap());
    }


    Ok(())
}

