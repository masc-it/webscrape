use webscrape::{ScraperBuilder, ScrapingPipeline};


pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    
    env_logger::init();
    let pipeline_file = std::env::args().nth(1).expect("no pipeline source given");

    println!("--------------------");

    let mut pipeline = ScrapingPipeline::from_file(&pipeline_file);
    
    print!("{}", pipeline);

    /* for step in &pipeline.get_steps() {
        println!("{}", step);
    } */
    let mut line = String::new();
    print!("Run : ");
    let b1 = std::io::stdin().read_line(&mut line).unwrap();

    let res = pipeline.run(&"DEFAULT".to_string());

    println!("num: {}", &res.elements.len());

    /* for (name, els) in res.elements {

        println!("TARGET: {}", name);
        for el in &els {
            println!("{:?}", el.text);

            for (k, v) in &el.attrs {
                println!("{} - {}", k, v);
            }
            println!("--------------------");
        }
    } */
    Ok(())
}
