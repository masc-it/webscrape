mod webscrape;

use crate::webscrape::{scraper::{ScraperBuilder, Builder}, proxy::{CSVProxyListBuilder, FromCSVBuilder, ProxyField}};


fn main() -> Result<(), Box<dyn std::error::Error>> {

    let mut csv_proxylist_builder = CSVProxyListBuilder::default();

    let proxies = csv_proxylist_builder
        .set_source("")
        .set_separator(":".to_string())
        .set_columns([ProxyField::Host, ProxyField::Port, ProxyField::Username, ProxyField::Password])
        .build();

    let mut builder = ScraperBuilder::default();

    let mut scraper = builder
        .set_headless(true)
        .set_default_timeout(5)
        .set_proxies(proxies)
        .build();

    /* let elements = scraper
        .navigate_to("https://httpbin.org/ip")
        .find_elements_by_css("body",  "body")
        .collect();

    for (name, el) in elements {
        println!("{:?}", el.text);
        
    } */

    println!("--------------------");

    let elements = scraper
        .navigate_to("https://stackoverflow.com/questions/58787864/changing-primary-palette-color-when-using-kivymd-has-no-effect-on-buttons")
        .find_elements_by_xpath("last_activity", "//a[contains(@href, 'lastact')]")
        .find_elements_by_xpath("question", "//h1/a[contains(@class, 'question-hyperlink')]")
        .find_elements_by_xpath("views", "//div[contains(@title, 'Viewed')]")
        .collect();

    for (name, el) in elements {
        println!("{:?}", el.text);

        println!("{:?}", el.attrs.get("id").unwrap_or(&"NONE".to_string()));

        for (k, v) in el.attrs {
            println!("{} - {}", k, v);
        }
    }
    
    // let elements = scraper
    //     .navigate_to("https://whatismycountry.com/")
    //     .find_elements_by_xpath("country", "//*[@id='country']")
    //     .collect();

    /* for (name, el) in elements {
        println!("{:?}", el.text);

        println!("{:?}", el.attrs.get("id").unwrap_or(&"NONE".to_string()));

        for (k, v) in el.attrs {
            println!("{} - {}", k, v);
        }
    } */

    

    Ok(())
}

