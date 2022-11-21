use webscrape::{ScraperBuilder, ScrapingPipeline, ScrapingResult, PipelineRunner};
use rayon::prelude::*;
use std::time::Instant;

fn main() {

    //rayon::ThreadPoolBuilder::new().num_threads(12).build_global().unwrap();

    let pipeline_file = std::env::args().nth(1).expect("no pipeline source given");

    let p1 = pipeline_file.clone();

    let mut sites: Vec<String> = vec![];
    
    let paths = std::fs::read_dir("/Users/maurosciancalepore/Downloads/pskm/html").unwrap();

    for path in paths {
        sites.push( format!("file://{}", path.unwrap().path().display().to_string()));
        //println!("Name: {}", path.unwrap().path().display())
    }

    /* for i in 0..5 {
        sites.push(format!("https://stackoverflow.com/questions?tab=newest&page={}&pagesize=50", i+1))
    } */

    let mut builder = ScraperBuilder::default();
    
    builder
        .set_headless(true)
        .set_default_timeout(15);

    let t0 = Instant::now();

    println!("Running...");
    let scraping_results = PipelineRunner::go(p1, builder, &sites);

    let t1 = t0.elapsed();

    println!("[Parallel] Done in {:.2?}.", t1);

    return;
    let t0 = Instant::now();
    let s = sites.iter().map(move |s| {

        let mut builder = ScraperBuilder::default();
        let scraper = builder.set_headless(true).set_default_timeout(5).build();
        
        let mut pipeline = ScrapingPipeline::from_file(&pipeline_file.clone(), scraper);
        let res = pipeline.run(&s);
        res

    }).collect::<Vec<ScrapingResult>>();

    let t1 = t0.elapsed();

    println!("[Serial] Done in {:.2?}.", t1);

    return; 
    for res in scraping_results {

        println!("RESULT");
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

    }
    
    /* let handle = tokio::spawn(async move {
        
            let mut builder = ScraperBuilder::default();
            let scraper = builder.set_headless(false).set_default_timeout(5).build();
            
            let mut pipeline = ScrapingPipeline::from_file(&pipeline_file.clone(), scraper);
            let res = pipeline.run(&"DEFAULT".to_string());

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
        let res = pipeline.run(&"DEFAULT".to_string());

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

    handle2.await.unwrap(); */
    
    
}

/* let mut line = String::new();
    print!("Run : ");
    let b1 = std::io::stdin().read_line(&mut line).unwrap(); */

    