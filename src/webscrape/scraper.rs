use std::{str::FromStr, sync::Arc, collections::HashMap};

use headless_chrome::{
    browser::{
        tab::RequestPausedDecision,
        transport::{SessionId, Transport},
    },
    protocol::cdp::{
        Fetch::{
            events::RequestPausedEvent, FailRequest, FulfillRequest, HeaderEntry
        },
        Network::ResourceType,
    },
    Browser, Element, LaunchOptions, Tab,
};
use rand::{rngs::StdRng, Rng, SeedableRng};

use super::proxy::SimpleProxy;

pub trait Builder {

    fn set_headless(&mut self, headless: bool) -> &mut ScraperBuilder;
    fn set_proxies(&mut self, proxies: Vec<SimpleProxy>) -> &mut ScraperBuilder;
    fn set_default_timeout(&mut self, default_timeout: u64) -> &mut ScraperBuilder;

    fn build(&self) -> Scraper;
}

pub struct ScraperBuilder {
    pub proxies: Vec<SimpleProxy>,
    pub default_timeout: u64,
    pub headless: bool
}

impl Default for ScraperBuilder {
    fn default() -> Self {
        Self { proxies: vec![], default_timeout: 5, headless: true }
    }
}

impl Builder for ScraperBuilder {

    fn set_headless(&mut self, headless: bool) -> &mut ScraperBuilder {
        self.headless = headless;
        self
    }
    fn set_default_timeout(&mut self, default_timeout: u64) -> &mut ScraperBuilder {
        self.default_timeout = default_timeout;
        self
    }

    fn set_proxies(&mut self, proxies: Vec<SimpleProxy>) -> &mut ScraperBuilder {
        self.proxies = proxies;
        self
    }

    fn build(&self) -> Scraper {
        let browser = Browser::new(LaunchOptions {
            //args: vec![&proxy_arg],
            headless: self.headless,
            ..Default::default()
        })
        .unwrap();

        let tab = browser.wait_for_initial_tab().unwrap();

        tab.set_default_timeout(std::time::Duration::from_secs(self.default_timeout));
        tab.enable_fetch(None, None).unwrap();

        let proxies = self.proxies.clone();
        tab.enable_request_interception(Arc::new(
            move |_: Arc<Transport>, _: SessionId, intercepted: RequestPausedEvent| {
                // !intercepted.params.request.url.ends_with(".jpg") && !intercepted.params.request.url.ends_with(".png") && !intercepted.params.request.url.ends_with(".js")
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

                    //println!("running reqwest.. {}", intercepted.params.request.url);
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
                } else {
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
        }
    }
}


#[derive(Clone)]
pub struct DOMElement {

    pub text: String,
    pub attrs: HashMap<String, String>
}

pub struct Scraper {
    pub proxy: Vec<SimpleProxy>,

    pub default_timeout: u64,

    browser: Browser,
    tab: Arc<Tab>,

    current_url: Option<String>,

    elements: HashMap<String, DOMElement>
}

impl Scraper {
    pub fn navigate_to<S: AsRef<str> + Clone>(&mut self, url: S) -> &mut Scraper {
        self.tab.navigate_to(url.as_ref()).unwrap();

        if let Err(_) = self.tab.wait_until_navigated() {
            println!("Page load timeout..");
        }

        self.tab
            .wait_for_xpath_with_custom_timeout("//body", std::time::Duration::from_secs(5))
            .unwrap();
        self.current_url = Some(url.as_ref().to_string());

        self
    }

    pub fn sleep(&self, seconds: u64) -> &Scraper {
        std::thread::sleep(std::time::Duration::from_secs(seconds));

        self
    }

    // TODO: list of lists or MAP
    // TODO: flatten()
    pub fn collect(&mut self) -> HashMap<String, DOMElement> {
        let r = self.elements.clone();
        self.elements.clear();
        r
    }

    pub fn find_elements_by_css<S: AsRef<str> + Clone>(&mut self, name: S, target: S) -> &mut Scraper 
    {
        let target = target.as_ref();
        let name = name.as_ref();

        if target.starts_with("/") {
            println!("Invalid CSS selector.");
            return self;
        }

        if let None = self.current_url {
            println!("Didn't you call navigate_to(url) ?");

            return self;
        }

        let query_result = self.tab.wait_for_elements(&target);

        let elements = match query_result {
            Ok(elements) => elements,
            Err(_) => vec![],
        };

        for el in &elements {

            self.elements.insert(name.to_string(), self.build_dom_element(el));
        }
        

        return self;
    }

    pub fn find_elements_by_xpath<S: AsRef<str> + Clone>(&mut self, name: S, target: S) -> &mut Scraper {
        
        let target = target.as_ref();
        let name = name.as_ref();

        if !target.starts_with("/") {
            println!("Invalid XPath selector.");
            return self;
        }

        if let None = self.current_url {
            println!("Didn't you call navigate_to(url) ?");

            return self;
        }

        let query_result = self.tab.wait_for_elements_by_xpath(&target);

        let elements = match query_result {
            Ok(elements) => elements,
            Err(_) => vec![],
        };

        for el in &elements {

            let dom_el = self.build_dom_element(el);
            self.elements.insert(name.to_string(), dom_el);
        }

        return self;
    }

    fn build_dom_element(&self, el: &Element ) -> DOMElement {

        let mut attrs_map: HashMap<String, String> = HashMap::default();

        let attrs = el.get_attributes().unwrap().unwrap();
        //println!("len {}", attrs.len());

        if attrs.len() > 0 {

            for i in (0..attrs.len() - 1).step_by(2) {
    
                let k = attrs[i].to_string();
                let v = attrs[i+1].to_string();
                attrs_map.insert(k, v);
            }
        }
        

        let dom_el = DOMElement {
            text: el.get_inner_text().unwrap(),
            attrs: attrs_map
        };

        dom_el
    }

}
