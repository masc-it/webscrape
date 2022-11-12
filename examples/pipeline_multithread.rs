use webscrape::{ScraperBuilder, ScrapingPipeline};
use std::thread;

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    
    let pipeline_file = std::env::args().nth(1).expect("no pipeline source given");

    for i in 1..10 {
        let pipeline_f = pipeline_file.clone();
        thread::spawn(move || {
        
            println!("hi number {} from the spawned thread!", i);
            let mut builder = ScraperBuilder::default();
            let scraper = builder.set_headless(false).set_default_timeout(5).build();
            
            let mut pipeline = ScrapingPipeline::from_file(&pipeline_f, scraper);
            let res = pipeline.run();

            println!("num: {}", &res.elements.len());
        
            for (name, els) in res.elements {
        
                println!("TARGET: {}", name);
                for el in &els {
                    println!("{:?}", el.text);
        
                    for (k, v) in &el.attrs {
                        println!("{} - {}", k, v);
                    }
                    println!("--------------------");
                }
            }
        });
    }

    let mut line = String::new();
    print!("Run : ");
    let b1 = std::io::stdin().read_line(&mut line).unwrap();
    
    Ok(())
}