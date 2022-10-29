use std::{io::{self, BufRead}, fs::File};


#[derive(Clone)]
pub struct SimpleProxy {

    pub host: String,
    pub port: String,
    pub user: String,
    pub password: String
}

impl SimpleProxy {
    
    pub fn get_address(&self) -> String {

        format!("http://{}:{}", self.host, self.port)

    }

    // https://proxy.webshare.io/proxy/list/download/exjkedupbocjavbsvygpqxginbabldofjdednvdb/-/http/username/direct/
    pub fn from_csv(proxy_list_source: String, split_on: String, host_pos: usize, port_pos: usize, user_pos: usize, pass_pos: usize) -> Vec<SimpleProxy>{

        let out = match proxy_list_source.starts_with("http") {

            true => {
                let mut resp = reqwest::blocking::get(&proxy_list_source).expect("request failed");
                let mut out = File::create("proxies.csv").unwrap();

                io::copy(&mut resp, &mut out).expect("failed to copy content");
                out.sync_all().unwrap();
                std::fs::File::open("proxies.csv").unwrap()

            },
            false => {
                std::fs::File::open(proxy_list_source).unwrap()
            }
            
        };

        let mut proxies: Vec<SimpleProxy> = vec![];
        
        let lines = std::io::BufReader::new(out).lines();

        for line in lines {
            if let Ok(proxy_string) = line {
                let parts = proxy_string.split(&split_on).collect::<Vec<&str>>();

                let p = SimpleProxy {
                    host: parts.get(host_pos).unwrap().to_string(),
                    port: parts.get(port_pos).unwrap().to_string(),
                    user: parts.get(user_pos).unwrap().to_string(),
                    password: parts.get(pass_pos).unwrap().to_string(),
                };

                proxies.push(p);

            }
        }


        proxies
    }
}
