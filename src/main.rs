use std::collections::HashMap;
use std::fs;
use std::net::ToSocketAddrs;
use std::process::exit;
use reqwest::blocking::Client;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
struct Config {
    email: String,
    password: String,
    pre_login_url: String,
    login_url: String,
    wifi_name: String,
    user_agent: String,
    email_type_map: HashMap<String, String>
}

impl Config {

    fn email_name(&self) -> &str {
        self.email.split("@").collect::<Vec<&str>>()[0]
    }

    fn email_tld(&self) -> &str {
        self.email.split("@").collect::<Vec<&str>>()[1]
    }

    fn email_type(&self) -> &str {
        let email_type = self.email_type_map.get(self.email_tld());
        match email_type {
            Some(email_type) => email_type,
            None => panic!("Unknown email type!")
        }
    }

    fn cache_builder(&self) -> String {
        format!("session=univ; loginUniv={}; mailusernameUniv={}; mailpassUniv={}; chkbxUniv=on; inet=false; loginUnivSuccess=true",
                self.email_type(),
                self.email_name(),
                self.password)
    }

}

impl Default for Config {
    fn default() -> Self {
        Config {
            email: String::from("email@binus.ac.id"),
            password: String::from("password"),
            pre_login_url: "https://backaccess.apps.binus.edu/wifi/loginValidator.php?=&prop=revisions&rvprop=content&format=json&callback=?&origin=*".to_string(),
            login_url: "https://access.apps.binus.ac.id/login".to_string(),
            wifi_name: String::from("Binus-Access"),
            user_agent: String::from("Mozilla/5.0 (X11; Linux x86_64; rv:109.0) Gecko/20100101 Firefox/110.0"),
            email_type_map: HashMap::from([
                (String::from("binus.ac.id"), String::from("ac.id")),
                (String::from("binus.org"), String::from("org")),
                (String::from("binus"), String::from(""))
            ])
        }
    }
}

fn main() {
    let config = load_config();
    let domain_ip = resolve_domain("access.apps.binus.ac.id:443")
        .or_else(|| resolve_domain("access.apps.binus.ac.id:80"));
    if domain_ip.is_none() {
        println!("Please connect to Binus-Access Wifi!");
        exit(0);
    }
    let start = std::time::Instant::now();
    let client = Client::new();
    let pre_login = pre_login(&client, &config);
    let login = login(&client, &config, &pre_login);
    if !login.contains("You are about to access the Internet Service operated by BINUS.") {
        println!("Login Success!");
    } else {
        println!("Login Failed!");
    }
    println!("Time elapsed: {:?} ms", start.elapsed().as_millis());
}

fn load_config() -> Config {
    let config_path = std::env::current_exe().unwrap().parent().unwrap().to_path_buf().join("config.ron");
    let path = config_path.to_str().unwrap();
    println!("Loading config...");
    if !std::path::Path::new(path).exists() {
        println!("Config file not found, creating new config file...");
        save_config(path, Config::default());
        println!("Config file created, please fill the config file with your email and password");
        exit(0);
    }
    let content = fs::read_to_string(path).expect("Something went wrong reading the file");
    let config = ron::de::from_str(&content).expect("Something went wrong parsing the file");
    save_config(path, config)
}

fn save_config(path: &str, config: Config) -> Config {
    let serialized = ron::ser::to_string_pretty(&config,
                                                ron::ser::PrettyConfig::default()).unwrap();
    fs::write(path, serialized).expect("Unable to write file");
    config
}

fn pre_login(client: &Client,config: &Config) -> String {
    println!("Getting user id...");
    let request_form =  [
        ("username", format!("{}@{}", config.email_name().to_uppercase(), config.email_tld())),
        ("password", format!("{}", config.password))
    ];

    let request = client.post(&config.pre_login_url)
        .form(&request_form)
        .header("User-Agent", &config.user_agent)
        .header("Accept", "*/*")
        .header("Accept-Language", "en-US,en;q=0.5")
        .header("Origin", "https://access.apps.binus.ac.id")
        .header("Referer", "https://access.apps.binus.ac.id/")
        .header("Sec-Fetch-Dest", "empty")
        .header("Sec-Fetch-Mode", "cors")
        .header("Sec-Fetch-Site", "cross-site")
        .header("TE", "trailers")
        .send()
        .expect("Failed to send request to get user id");

    let content = request.text().expect("Failed to get response from request to get user id");
    if content.starts_with("B"){
        return content
    } else {
        println!("Login Failed, please check your credentials!");
        exit(0)
    }
}

fn login(client: &Client, config: &Config, user_id: &str) -> String {
    println!("Logging in...");
    let cache = config.cache_builder();
    let request_form =  [
        ("dst", ""),
        ("popup", "false"),
        ("username", user_id),
        ("password", &config.password)
    ];

    let request = client.post(&config.login_url)
        .form(&request_form)
        .header("User-Agent", &config.user_agent)
        .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8")
        .header("Accept-Language", "en-US,en;q=0.5")
        .header("Cookie", &cache)
        .header("DNT", "1")
        .header("Origin", "https://access.apps.binus.ac.id")
        .header("Referer", "https://access.apps.binus.ac.id/login")
        .header("Upgrade-Insecure-Requests", "1")
        .header("Sec-Fetch-Dest", "document")
        .header("Sec-Fetch-Mode", "navigate")
        .header("Sec-Fetch-Site", "none")
        .header("Sec-Fetch-User", "1")
        .send()
        .expect("Failed to send request to login");

    request.text().expect("Failed to get response from login")
}

fn resolve_domain(domain: &str) -> Option<String> {
    let addrs_iter = domain.to_socket_addrs();
    if addrs_iter.is_err() {
        return None
    }
    Some(addrs_iter.unwrap().next().unwrap().ip().to_string())
}
