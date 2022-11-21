use std::{str::FromStr, sync::Arc, collections::HashMap, path::{PathBuf}};

use headless_chrome::{
    browser::{
        tab::RequestPausedDecision,
        transport::{SessionId, Transport},
    },
    protocol::{cdp::{
        Fetch::{
            events::RequestPausedEvent, FailRequest, FulfillRequest, HeaderEntry
        },
        Network::ResourceType
    }, self},
    Browser, Element, LaunchOptions, Tab,
};

use rand::{rngs::StdRng, Rng, SeedableRng};
use serde::Serialize;

use crate::proxy::SimpleProxy;


#[derive(Clone, Serialize)]
/// It contains all the metadata of a scraped element.
pub struct DOMElement {
    
    pub text: String,
    pub attrs: HashMap<String, String>
}

enum Selector {
    CSS,
    XPath
}

pub enum ScreenshotFormat {
    JPEG,
    PNG
}

/// Scraper is the main player of this crate. <br>
/// It wraps a Chrome browser and high level interfaces to scrape DOM elements and run automated actions.
pub struct Scraper {
    pub proxy: Vec<SimpleProxy>,

    pub default_timeout: u64,

    browser: Browser,
    tab: Arc<Tab>,

    current_url: Option<String>,

    elements: HashMap<String, Vec<DOMElement>>,

    screenshots: HashMap<String, Vec<u8>>,

    save_dir: String
}

/// Just a builder for the Scraper struct. <br>
/// You can use it to build new Scraper instances.
/// 
#[derive(Clone)]
pub struct ScraperBuilder {
    pub proxies: Vec<SimpleProxy>,
    pub default_timeout: u64,
    pub headless: bool,
    pub save_dir: String
}

impl Default for ScraperBuilder {
    fn default() -> Self {

        let save_dir = ".scraping_results/".to_string();
        Self { proxies: vec![], default_timeout: 5, headless: true, save_dir: save_dir.clone() }
    }
}


pub struct ScrapingResult {

    pub elements: HashMap<String, Vec<DOMElement>>,
    pub screenshots: HashMap<String, Vec<u8>>
}

impl ScraperBuilder {

    pub fn set_headless(&mut self, headless: bool) -> &mut ScraperBuilder {
        self.headless = headless;
        self
    }
    pub fn set_default_timeout(&mut self, default_timeout: u64) -> &mut ScraperBuilder {
        self.default_timeout = default_timeout;
        self
    }

    pub fn set_proxies(&mut self, proxies: Vec<SimpleProxy>) -> &mut ScraperBuilder {
        self.proxies = proxies;
        self
    }

    pub fn set_save_dir(&mut self, save_dir: String) -> &mut ScraperBuilder {
        self.save_dir = save_dir;
        self
    }

    /// It materializes a new Scraper instance with the provided properties.
    pub fn build(&self) -> Scraper {
       
        std::fs::create_dir(&self.save_dir).unwrap_or(());

        let browser_path = match std::env::consts::OS {
            "windows" => "%ProgramFiles%/Google/Chrome/Application/chrome.exe",
            "macos" => "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
            _ => ""
        };

        let browser = Browser::new(LaunchOptions {
            path: Some(PathBuf::from_str(browser_path).unwrap()),
            headless: self.headless,
            ..Default::default()
        })
        .unwrap();

        let tab = browser.wait_for_initial_tab().unwrap();

        tab.set_default_timeout(std::time::Duration::from_secs(self.default_timeout));
        tab.enable_fetch(None, None).unwrap();

        let proxies = self.proxies.clone();

       /*  let sync_event = Arc::new(move |event: &Event| match event {
            Event::PageLifecycleEvent(lifecycle) => {
                if lifecycle.params.name == "DOMContentLoaded" {
                    println!("{}", "loaded");
                }
            }
            _ => {}
        });
    
        tab.add_event_listener(sync_event).unwrap(); */

        /* 
        Tab interception is useful to:
            - Enable request-level proxying
            - Block requests based on mime type  
        */
        tab.enable_request_interception(Arc::new(
            move |_: Arc<Transport>, _: SessionId, intercepted: RequestPausedEvent| {
                // !intercepted.params.request.url.ends_with(".jpg") && !intercepted.params.request.url.ends_with(".png") && !intercepted.params.request.url.ends_with(".js")
                
                if intercepted.params.request.url.starts_with("file:///") {
                    return RequestPausedDecision::Continue(None);
                }
                let v = intercepted.params.request.headers.0.unwrap();

                let headersmap = v.as_object().unwrap().clone();

                //println!("{:?}", intercepted.params.resource_Type);

                if intercepted.params.resource_Type == ResourceType::Document {
                    
                    let mut reqwest_proxy: Option<reqwest::Proxy> = None;
                    if proxies.len() > 0 {
                        let mut rng: StdRng = SeedableRng::from_entropy();
                        let rnd_i = rng.gen_range(0..proxies.len());
                        let proxy = proxies.get(rnd_i).unwrap().to_owned();
                        //println!("{}", proxy.get_address());

                        reqwest_proxy = match proxy.user == "" {
                            true => Some(reqwest::Proxy::all(proxy.get_address()).unwrap()),
                            false => Some(
                                        reqwest::Proxy::all(proxy.get_address())
                                            .unwrap()
                                            .basic_auth(&proxy.user, &proxy.password),
                                    )
                        };
                    }

                    let mut req_headers: Vec<HeaderEntry> = vec![];

                    // Build a header map to simulate a real browser request. 
                    let mut headers = reqwest::header::HeaderMap::new();

                    for (k, val) in headersmap {
                        let myk = k.to_owned();
                        let myv = val.to_owned();
                        let myv = myv.to_string();

                        let key = reqwest::header::HeaderName::from_str(&myk).unwrap();
                        let vall = reqwest::header::HeaderValue::from_str(&myv);

                        headers.insert(key, vall.unwrap());
                    }

                    req_headers.push(HeaderEntry {
                        name: "Content-Type".to_string(),
                        value: "text/html; charset=utf-8".to_string(),
                    });

                    let req_builder = reqwest::blocking::Client::builder();

                    let req_builder = match reqwest_proxy {
                        Some(p) => req_builder.proxy(p),
                        None => req_builder,
                    };

                    let client = req_builder
                        .default_headers(headers.to_owned())
                        .build()
                        .unwrap();

                    let res = client
                        .get(intercepted.params.request.url)
                        .send()
                        .unwrap()
                        .text()
                        .unwrap();

                    let fulfill_request = FulfillRequest {
                        request_id: intercepted.params.request_id,
                        response_code: 200,
                        response_headers: Some(req_headers),
                        binary_response_headers: None,
                        body: Some(base64::encode(res)),
                        response_phrase: None,
                    };

                    RequestPausedDecision::Fulfill(fulfill_request)
                } else if intercepted.params.resource_Type != ResourceType::Document {
                    RequestPausedDecision::Continue(None)
                } else { // TODO: block some resources
                    RequestPausedDecision::Fail(FailRequest {
                        request_id: intercepted.params.request_id,
                        error_reason:
                            headless_chrome::protocol::cdp::Network::ErrorReason::BlockedByClient,
                    })
                }
            },
        )).expect("You should check the validity of your proxies or the URL provided.");

        Scraper {
            proxy: self.proxies.clone(),
            default_timeout: self.default_timeout,
            browser,
            tab,
            current_url: None,
            elements: HashMap::default(),
            screenshots: HashMap::default(),
            save_dir: self.save_dir.clone()
        }
    }
}




impl Scraper {
    pub fn navigate_to<S: AsRef<str> + Clone>(&mut self, url: S) -> &mut Scraper {
        self.tab.navigate_to(url.as_ref()).unwrap();

        if let Err(_) = self.tab.wait_until_navigated() {
            //println!("Page load timeout..");
        }

        if let Err(_) = self.tab
            .wait_for_xpath_with_custom_timeout("//body", std::time::Duration::from_secs(5)){
                println!("Page load timeout..");
            }
        self.current_url = Some(url.as_ref().to_string());

        self
    }

    pub fn sleep(&self, seconds: u64) -> &Scraper {
        std::thread::sleep(std::time::Duration::from_secs(seconds));

        self
    }

    pub fn collect(&mut self) -> ScrapingResult {

        let res = ScrapingResult {
            elements: self.elements.clone(),
            screenshots: self.screenshots.clone()
        };
        
        self.elements.clear();
        self.screenshots.clear();
        res
    }

    pub fn find_elements_by_css<S: AsRef<str> + Clone>(&mut self, name: S, target: S) -> &mut Scraper 
    {
        let target = target.as_ref();
        let name = name.as_ref();

        if target.starts_with("/") {
            println!("Invalid CSS selector: {}", target);
            return self;
        }

        if let None = self.current_url {
            println!("Didn't you call navigate_to(url) ?");

            return self;
        }

        let query_result = self.tab.wait_for_elements(&target);

        let elements = match query_result {
            Ok(elements) => elements,
            Err(_) => { vec![]}, // println!("Element {} not found", name);
        };

        let dom_els: Vec<DOMElement> = elements.iter().map(|el| self.build_dom_element(el)).collect();

        self.elements.insert(name.to_string(), dom_els);
        

        return self;
    }

    pub fn find_elements_by_xpath<S: AsRef<str> + Clone>(&mut self, name: S, target: S) -> &mut Scraper {
        
        let target = target.as_ref();
        let name = name.as_ref();

        if !target.starts_with("/") {
            println!("Invalid XPath selector: {}", target);
            return self;
        }

        if let None = self.current_url {
            println!("Didn't you call navigate_to(url) ?");

            return self;
        }

        /* self.tab.reload(false, None).unwrap();

        self.tab.wait_until_navigated().unwrap();
        let r = self.tab.wait_for_xpath_with_custom_timeout("//body", std::time::Duration::from_secs(5)).unwrap();
 */

        let query_result = self.tab.wait_for_elements_by_xpath(&target);

        let elements = match query_result {
            Ok(elements) => elements,
            Err(_) => {vec![]}, //println!("Element {} not found", name);
        };

        let dom_els: Vec<DOMElement> = elements.iter().map(|el| self.build_dom_element(el)).collect();

        self.elements.insert(name.to_string(), dom_els);
        
        return self;
    }

    fn build_dom_element(&self, el: &Element ) -> DOMElement {

        let mut attrs_map: HashMap<String, String> = HashMap::default();

        let attrs = el.get_attributes().unwrap().unwrap();

        let tag = el.get_description().unwrap().local_name;
        if attrs.len() > 0 {

            for i in (0..attrs.len() - 1).step_by(2) {
    
                let k = attrs[i].to_string();
                let v = attrs[i+1].to_string();
                attrs_map.insert(k, v);
            }
        }

        attrs_map.insert("tag".to_string(), tag);
        

        let dom_el = DOMElement {
            text: el.get_inner_text().unwrap(),
            attrs: attrs_map
        };

        dom_el
    }

    fn get_selector_type<S: AsRef<str> + Clone>(&self, target: &S) -> Selector {

        let target = target.as_ref();
        if target.starts_with("/") {
            return Selector::XPath;
        }

        Selector::CSS
    }

    pub fn click<S: AsRef<str> + Clone>(&mut self, name: S, target: S) -> &mut Scraper {

        let target = target.as_ref();
        let selector_type = self.get_selector_type(&target);

        let res = match selector_type {
            Selector::CSS => self.tab.wait_for_element(&target).and_then(|el| {el.click().and_then(|_r| Ok(true))}),
            Selector::XPath => self.tab.wait_for_elements_by_xpath(&target).and_then(|el| {el.get(0).unwrap().click().and_then(|_r| Ok(true))}),
        };

        if res.is_err() {
            println!("Couldn't find element: {}", name.as_ref());
        }

        if let Err(_) = self.tab.wait_until_navigated() {
            println!("Page load timeout..");
        }
        
        let r = self.tab.wait_for_xpath_with_custom_timeout("//body", std::time::Duration::from_secs(5));

        if r.is_err() {
            println!("Page load timeout..");
        }


        self
    }

    pub fn type_into<S: AsRef<str> + Clone>(&mut self, name: S, target: S, text: S) -> &mut Scraper {

        let target = target.as_ref();
        let name = name.as_ref();
        let text = text.as_ref();

        let selector_type = self.get_selector_type(&target);

        let res = match selector_type {
            Selector::CSS => self.tab.wait_for_element(&target).and_then(|el| {el.type_into(&text).and_then(|_r| Ok(true))}),
            Selector::XPath => self.tab.wait_for_elements_by_xpath(&target).and_then(|el| {el.get(0).unwrap().type_into(&text).and_then(|_r| Ok(true))}),
        };

        if res.is_err() {
            println!("Couldn't find element: {}", name);
        }

        self
    }

    pub fn screenshot<S: AsRef<str> + Clone>(&mut self, name: S, target: S, format: ScreenshotFormat) -> &mut Scraper {

        let target = target.as_ref();
        let name = name.as_ref();

        let format = match format {
            ScreenshotFormat::JPEG => protocol::cdp::Page::CaptureScreenshotFormatOption::Jpeg,
            ScreenshotFormat::PNG => protocol::cdp::Page::CaptureScreenshotFormatOption::Png,
            _ => protocol::cdp::Page::CaptureScreenshotFormatOption::Png
        };

        let selector_type = self.get_selector_type(&target);

        let res = match selector_type {
            Selector::CSS => self.tab.wait_for_element(&target).and_then(|el| {el.capture_screenshot(format).and_then(|r| Ok(r))}),
            Selector::XPath => self.tab.wait_for_elements_by_xpath(&target).and_then(|el| {el.get(0).unwrap().capture_screenshot(format).and_then(|r| Ok(r))}),
        };

        let Ok(img_data) = res else {

            println!("Couldn't find element: {}", name);
            return self;
        };

        self.screenshots.insert(name.to_string(), img_data);
        

        self
    }


    pub fn save(&self, targets: &Vec<String>, flatten: &bool ) {

        let curr_url = self.current_url.as_ref().unwrap();
        let parts = curr_url.split("/").collect::<Vec<&str>>();
        let name = parts.last().unwrap().to_string();
        let name = name.split_once(".").unwrap_or((name.as_str(), ""));
        let name = name.0.to_string();

        let save_path = &self.save_dir;

        let save_path = format!("{}/{}.json", save_path, name);

        let mut els = self.elements.clone();
        els.retain(|k,_| targets.contains(k));

        if *flatten {

            let els = els.iter().map(|(k,v)| v.clone() ).collect::<Vec<Vec<DOMElement>>>();
            let els = els.into_iter().flatten().collect::<Vec<DOMElement>>();
            let s = serde_json::to_string_pretty(&els).unwrap();

            std::fs::write(save_path, s).unwrap();
        } else {
            let s = serde_json::to_string_pretty(&els).unwrap();

            std::fs::write(save_path, s).unwrap();
        }
        
        
    }
}
