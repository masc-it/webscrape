use webscrape::{ScraperBuilder, ScrapingPipeline};
use std::thread;

#[tokio::main]
async fn main() {

    let pipeline_file = std::env::args().nth(1).expect("no pipeline source given");

    let p1 = pipeline_file.clone();

    let handle = tokio::spawn(async move {
        
            let mut builder = ScraperBuilder::default();
            let scraper = builder.set_headless(false).set_default_timeout(5).build();
            
            let mut pipeline = ScrapingPipeline::from_file(&pipeline_file.clone(), scraper);
            let res = pipeline.run();

            println!("YO BY 1");
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

    let handle2 = tokio::spawn(async move {
        
        let mut builder = ScraperBuilder::default();
        let scraper = builder.set_headless(false).set_default_timeout(5).build();
        
        let mut pipeline = ScrapingPipeline::from_file(&p1, scraper);
        let res = pipeline.run();

        println!("YO BY 2");
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

    // Do some other work

    let out = handle.await.unwrap();

    handle2.await.unwrap();
    
    
}

/* let mut line = String::new();
    print!("Run : ");
    let b1 = std::io::stdin().read_line(&mut line).unwrap(); */

    