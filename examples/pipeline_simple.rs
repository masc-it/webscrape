use webscrape::{ScraperBuilder, ScrapingPipeline};


pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    
    let pipeline_file = std::env::args().nth(1).expect("no pipeline source given");

    let mut builder = ScraperBuilder::default();

    let scraper = builder.set_headless(true).set_default_timeout(5).build();

    println!("--------------------");

    let pipeline = ScrapingPipeline::from_file(&pipeline_file, scraper);
    
    print!("{}", pipeline);

    Ok(())
}
