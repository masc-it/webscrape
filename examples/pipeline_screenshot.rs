use std::path::Path;

use webscrape::{ScraperBuilder, ScrapingPipeline, save_screenshot};


pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    
    let pipeline_file = std::env::args().nth(1).expect("no pipeline source given");

    let mut builder = ScraperBuilder::default();

    let scraper = builder.set_headless(false).set_default_timeout(5).build();

    println!("--------------------");

    let mut pipeline = ScrapingPipeline::from_file(&pipeline_file, scraper);
    
    print!("{}", pipeline);

    
    
    let mut line = String::new();
    print!("Run : ");
    let b1 = std::io::stdin().read_line(&mut line).unwrap();

    let res = pipeline.run();

    println!("num: {}", &res.screenshots.len());


    for (name, img_data) in res.screenshots {

        println!("SCREENSHOT: {}", name);
        let s_name = format!("{}.png", &name);
        save_screenshot(&img_data, &Path::new(&s_name)).unwrap();

    }
    Ok(())
}
