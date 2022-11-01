use std::{io::{self, BufRead}, fs::File};

#[derive(Eq, Hash, PartialEq, Default)]
pub enum ProxyField {
    #[default] Host,
    Port,
    Username,
    Password
}


pub struct CSVProxyListBuilder {

    source: String,
    fields: [ProxyField; 4],
    separator: String,

}

impl Default for CSVProxyListBuilder {

    fn default() -> Self {
        Self { source: String::from("./config/proxies.csv"), fields: [ProxyField::Host, ProxyField::Port, ProxyField::Username, ProxyField::Password], separator: String::from(",") }
    }
}

impl CSVProxyListBuilder {
    pub fn set_source<S: AsRef<str> + Clone>(&mut self, file_or_url: S) -> &mut CSVProxyListBuilder {
        self.source = file_or_url.as_ref().to_string();

        self
    }

    pub fn set_separator<S: AsRef<str> + Clone>(&mut self, separator: S) -> &mut CSVProxyListBuilder {
        self.separator = separator.as_ref().to_string();
        self
    }

    pub fn set_columns(&mut self, columns: [ProxyField; 4]) -> &mut CSVProxyListBuilder {
        self.fields = columns;
        self
    }

    pub fn build(&self) -> Vec<SimpleProxy> {
        
        let source_file = match self.source.starts_with("http") {

            true => {
                let mut resp = reqwest::blocking::get(&self.source).expect("Couldn't download the proxy list. You should provide a valid Proxy URL.");
                
                std::fs::create_dir("config/").unwrap_or(());
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

}
