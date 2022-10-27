use super::proxy::Proxy;

pub struct Scraper {

    pub persistent: bool,
    pub proxy: Proxy
}

impl Scraper {

    fn load_url(url: String, render_js: bool) -> String {
        "".to_owned()
    }

}