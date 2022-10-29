use std::{sync::Arc, str::FromStr};

use headless_chrome::{Browser, LaunchOptions, Tab, Element, browser::{transport::{Transport, SessionId}, tab::RequestPausedDecision}, protocol::cdp::{Fetch::{events::RequestPausedEvent, HeaderEntry, FulfillRequest, RequestPattern, RequestStage, FailRequest}, Network::ResourceType}};

use super::{proxy::Proxy, pipeline::Target};

pub struct Scraper {

    pub init_on_create: bool,
    pub persistent: bool,
    pub proxy: Vec<Proxy>,

    pub default_timeout: u64,

    browser: Browser,
    tab: Arc<Tab>,

    current_url: Option<String>
}

impl Default for Scraper {
    fn default() -> Self {

        //let proxy_url = format!("--proxy-server=http://{}:{}", "37.35.43.159", "9017");
        //let proxy_arg = std::ffi::OsString::from(proxy_url);
        let browser = Browser::new(LaunchOptions{

            //args: vec![&proxy_arg],
            headless: false,
            ..Default::default()
        }).unwrap();
        
        let tab = browser.wait_for_initial_tab().unwrap();
        
        tab.set_default_timeout(std::time::Duration::from_secs(5));
        tab.enable_fetch(None, None).unwrap();

        tab.enable_request_interception(Arc::new(
            move |transport: Arc<Transport>, session_id: SessionId, intercepted: RequestPausedEvent| {
                
                //println!("{:?}", intercepted.params.resource_Type);
                // !intercepted.params.request.url.ends_with(".jpg") && !intercepted.params.request.url.ends_with(".png") && !intercepted.params.request.url.ends_with(".js")
                let v = intercepted.params.request.headers.0.unwrap();

                let headersmap = v.as_object().unwrap().clone();

                println!("{:?}", intercepted.params.resource_Type);

                if intercepted.params.resource_Type == ResourceType::Document {
                let proxy = format!("http://{}:{}", "5.154.254.0", "5011");
                
                
                let mut req_headers : Vec<HeaderEntry> = vec![];

                let mut headers = reqwest::header::HeaderMap::new();

                for (k,val) in headersmap{

                    //println!("{} - {}", &k, &val.as_str().unwrap());
                    /* req_headers.push(HeaderEntry {
                        name: k.to_owned(),
                        value: val.as_str().unwrap().to_owned()
                    }); */
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
                println!("running reqwest.. {}", intercepted.params.request.url);
                let res = reqwest::blocking::Client::builder()
                    .proxy(
                        reqwest::Proxy::all(&proxy)
                            .unwrap()
                            .basic_auth("proxygobrr69", "dksfjlfnajd32429"),
                    )
                    .default_headers(headers.to_owned())
                    .build()
                    .unwrap()
                    .get(intercepted.params.request.url).send().unwrap().text().unwrap();
                    
    
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
                } else {
                    RequestPausedDecision::Fail(FailRequest { request_id: intercepted.params.request_id, error_reason: headless_chrome::protocol::cdp::Network::ErrorReason::BlockedByClient })
                    
                }
            },
        )).unwrap();
        Self { init_on_create: false, persistent: false, proxy: vec![], browser: browser, default_timeout: 5, tab: tab, current_url: None }
    }
}

impl Scraper {

    pub fn navigate_to(&mut self, url: &String) -> bool {

        self.tab.navigate_to(url).unwrap();

        self.tab.wait_until_navigated().unwrap();

        self.tab.wait_for_xpath_with_custom_timeout("//body", std::time::Duration::from_secs(5)).unwrap();
        self.current_url = Some(url.to_string());
        
        return true;
        
    }

    pub fn sleep(&mut self, seconds: u64) {

        std::thread::sleep(std::time::Duration::from_secs(seconds));
        
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