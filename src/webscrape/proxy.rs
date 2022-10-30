use std::{io::{self, BufRead}, fs::File, collections::HashMap};

#[derive(Eq, Hash, PartialEq, Default)]
pub enum ProxyField {
    #[default] Host,
    Port,
    Username,
    Password
}

pub trait FromCSVBuilder {

    fn set_source(&mut self, file_or_url: String) -> &mut CSVProxyListBuilder;

    fn set_columns(&mut self, columns: [ProxyField; 4]) -> &mut CSVProxyListBuilder;
    fn set_field_pos(&mut self, field: ProxyField, pos: u32) -> &mut CSVProxyListBuilder;

    fn set_separator(&mut self, separator: String) -> &mut CSVProxyListBuilder;

    fn build(&self) -> Vec<SimpleProxy>;
}

#[derive(Default)]
pub struct CSVProxyListBuilder {

    source: String,

    parts_pos: HashMap<ProxyField, usize>,
    fields: [ProxyField; 4],
    separator: String,

}

impl FromCSVBuilder for CSVProxyListBuilder {
    fn set_source(&mut self, file_or_url: String) -> &mut CSVProxyListBuilder {
        self.source = file_or_url;

        self
    }

    fn set_separator(&mut self, separator: String) -> &mut CSVProxyListBuilder {
        self.separator = separator;
        self
    }

    fn set_field_pos(&mut self, field: ProxyField, pos: u32) -> &mut CSVProxyListBuilder {
        
        
        self.parts_pos.insert(field, pos as usize);
        self

    }

    fn set_columns(&mut self, columns: [ProxyField; 4]) -> &mut CSVProxyListBuilder {
        self.fields = columns;
        self
    }

    fn build(&self) -> Vec<SimpleProxy> {
        
        let source_file = match self.source.starts_with("http") {

            true => {
                let mut resp = reqwest::blocking::get(&self.source).expect("Couldn't download the proxy list. You should provide a valid Proxy URL.");
                
                let mut out = File::create("config/proxies.csv").unwrap();

                io::copy(&mut resp, &mut out).expect("failed to copy content");
                out.sync_all().unwrap();
                std::fs::File::open("config/proxies.csv").unwrap()

            },
            false => {
                std::fs::File::open(&self.source).expect(format!("Proxy file not found: {}", &self.source).as_str())
            }
            
        };

        let host_pos = self.fields.iter().position(|e| *e == ProxyField::Host).expect("Port column should be defined.");
        //let host_pos = self.parts_pos.get(&ProxyField::Host).expect("Host position should be defined.").to_owned();
        //let port_pos = self.parts_pos.get(&ProxyField::Port).expect("Port position should be defined.").to_owned();
        
        let port_pos = self.fields.iter().position(|e| *e == ProxyField::Port).expect("Port column should be defined.");

        let user_pos = self.fields.iter().position(|e| *e == ProxyField::Username).expect("Username column should be defined.");
        
        let password_pos = self.fields.iter().position(|e| *e == ProxyField::Password).expect("Password column should be defined.");
        
        let mut proxies: Vec<SimpleProxy> = vec![];
        
        let lines = std::io::BufReader::new(&source_file).lines();

        for line in lines {
            if let Ok(proxy_string) = line {
                let parts = proxy_string.split(&self.separator).collect::<Vec<&str>>();

                let p = SimpleProxy {
                        host: parts.get(host_pos).unwrap().to_string(),
                        port: parts.get(port_pos).unwrap().to_string(),
                        user: parts.get(user_pos).unwrap().to_string(),
                        password: parts.get(password_pos).unwrap().to_string(),
                    };

                proxies.push(p);

            }
        }

        proxies

    }
}

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
