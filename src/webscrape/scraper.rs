use std::sync::Arc;

use headless_chrome::{Browser, LaunchOptions, Tab, Element, browser::{transport::{Transport, SessionId}, tab::RequestPausedDecision}, protocol::cdp::{Fetch::{events::RequestPausedEvent, HeaderEntry, FulfillRequest, RequestPattern, RequestStage}, Network::ResourceType}};

use super::{proxy::Proxy, pipeline::Target};

pub struct Scraper {

    pub init_on_create: bool,
    pub persistent: bool,
    pub proxy: Vec<Proxy>,

    browser: Browser,
    tab: Arc<Tab>,

    current_url: Option<String>
}

impl Default for Scraper {
    fn default() -> Self {

        let browser = Browser::new(LaunchOptions{

            //args: vec![&proxy_arg],
            headless: false,
            ..Default::default()
        }).unwrap();
        
        let tab = browser.wait_for_initial_tab().unwrap().to_owned();

        let patterns = vec![
            RequestPattern {
                url_pattern: None,
                resource_Type: None,
                request_stage: Some(RequestStage::Response),
            },
            RequestPattern {
                url_pattern: None,
                resource_Type: None,
                request_stage: Some(RequestStage::Request),
            },
        ];
        tab.enable_fetch(Some(&patterns), None).unwrap();

        tab.enable_request_interception(Arc::new(
            move |transport: Arc<Transport>, session_id: SessionId, intercepted: RequestPausedEvent| {
                
                //println!("{:?}", intercepted.params.resource_Type);
                // !intercepted.params.request.url.ends_with(".jpg") && !intercepted.params.request.url.ends_with(".png") && !intercepted.params.request.url.ends_with(".js")
                if intercepted.params.resource_Type == ResourceType::Document {
                let proxy = format!("http://{}:{}", "45.192.146.173", "6184");
                
                println!("running reqwest.. {}", intercepted.params.request.url);
                let res = reqwest::blocking::Client::builder()
                    .proxy(
                        reqwest::Proxy::all(&proxy)
                            .unwrap()
                            .basic_auth("proxygobrr69", "dksfjlfnajd32429"),
                    )
                    .build()
                    .unwrap()
                    .get(intercepted.params.request.url).send().unwrap().text().unwrap();
                    
                    let headers = vec![HeaderEntry {
                        name: "Content-Type".to_string(),
                        value: "text/html; charset=utf-8".to_string(),
                    }];
    
                    let fulfill_request = FulfillRequest {
                        request_id: intercepted.params.request_id,
                        response_code: 200,
                        response_headers: Some(headers),
                        binary_response_headers: None,
                        body: Some(base64::encode(res)),
                        response_phrase: None,
                    };
    
                    RequestPausedDecision::Fulfill(fulfill_request)
                } else {
                    RequestPausedDecision::Continue(None)
                }
            },
        )).unwrap();
        Self { init_on_create: false, persistent: false, proxy: vec![], browser: browser, tab: tab, current_url: None }
    }
}

impl Scraper {

    pub fn navigate_to(&mut self, url: &String) -> bool {

        self.tab.navigate_to(url).unwrap();

        self.tab.wait_until_navigated().unwrap();

        self.current_url = Some(url.to_string());
        
        return true;
        
    }

    pub fn find_elements_by_css(&self, target: &String ) -> Vec<Element> {
        
        if target.starts_with("/") {
            println!("Invalid CSS selector.");
            return vec![];
        }

        if let None = self.current_url {
            println!("Didn't you call navigate_to(url) ?");

            return vec![];
        }

        let query_result = self.tab.wait_for_elements(&target);

        let elements = match query_result {
            Ok(elements) => elements,
            Err(_) => vec![]
        };

        return elements;

    }

    pub fn find_elements_by_xpath(&self, target: &String ) -> Vec<Element> {
        
        if !target.starts_with("/") {
            println!("Invalid XPath selector.");
            return vec![];
        }

        if let None = self.current_url {
            println!("Didn't you call navigate_to(url) ?");

            return vec![];
        }

        let query_result = self.tab.wait_for_elements_by_xpath(&target);

        let elements = match query_result {
            Ok(elements) => elements,
            Err(_) => vec![]
        };

        return elements;

    }

}