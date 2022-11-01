use webscrape::{ScraperBuilder};


pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut builder = ScraperBuilder::default();

    let mut scraper = builder.set_headless(true).set_default_timeout(5).build();

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

    Ok(())
}
