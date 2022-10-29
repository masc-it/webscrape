
fn teststuff(){


let f = std::fs::File::open("config_ip.yaml")?;

    let obj: webscrape::pipeline::PipelineConfig = serde_yaml::from_reader(f).unwrap();

    let targets = &obj.targets;

    let proxy_url = format!("--proxy-server=http://{}:{}", "45.192.141.97", "5134"); //`http://${this.randomProxy.hostname}:${this.randomProxy.port}`
    
    let no_sandbox = "--no-sandbox";
    let proxy_arg = std::ffi::OsString::from(proxy_url);

    let sandbox_arg = std::ffi::OsString::from(no_sandbox);
    let browser = Browser::new(LaunchOptions{

        args: vec![&proxy_arg],
        headless: false,
        ..Default::default()
    }).unwrap();
    
    let tab = &browser.wait_for_initial_tab()?;

    tab.enable_fetch(None, Some(true)).unwrap();
    tab.authenticate(Some(String::from("proxygobrr69")), Some(String::from("dksfjlfnajd32429"))).unwrap();

    tab.navigate_to(&obj.pipeline.url).unwrap();

    tab.wait_until_navigated().unwrap();
    /* let el = tab.find_element_by_xpath("//a[contains(@href, 'lastact')]").unwrap();

    println!("{}", el.get_inner_text().unwrap()); */

    for t in targets.to_owned() {

        tab.wait_for_elements_by_xpath(&t.selector).unwrap();
        let r = t.find_element(tab);

        println!("{}", r);
    }
    /* for a in &actions {
        let d = &a.data;
        
        match d {
            // TODO try to flatten action payload in pipeline.rs
            ActionData::ActionClick(e) => e.run(tab),
            ActionData::ActionWait(e) => e.run(tab)
        };

    } */
   
    
    //println!("{:?}", obj.scraping_actions.)
    /* let s = webscrape::scraper::Scraper{

        persistent: true,
        proxy: Proxy{url: "".to_owned()}
    };
    
    let resp = reqwest::blocking::get("https://download.geofabrik.de/europe.html")?
        .text()?;
        
    let mut urls : Vec<(String, String)> = Vec::default();
    
    let fragment = Html::parse_fragment(&resp);
    let selector = Selector::parse("a[href*=\"-latest.osm.pbf\"]").unwrap();

    for element in fragment.select(&selector) {

        let el = element.value();
        
        let url = el.attr("href").unwrap().to_owned();

        let url_parts: Vec<_> = el.attr("href").unwrap().split("/").collect();
        if url_parts.len() < 2 {
            continue;
        }

        let name = url_parts.get(1).unwrap().to_owned().to_owned();
        urls.push((name, url));
    }
    
    for (name, url) in &urls {
        println!("{} - {}", name, url)
    } */

}