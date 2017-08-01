extern crate rand;
extern crate cookie;

use self::rand::random;
use self::cookie::Cookie;
use nickel::{Nickel, HttpRouter, FormBody};
use nickel::extensions::Redirect;
use nickel::Request;

use std::collections::HashMap;
use std::sync::{Arc,Mutex};
use ldap::Ldap;

use std::time::{Duration, Instant};

pub struct Client {
    sid: String,
    session: Instant, 
}

pub type Clients = Arc<Mutex<HashMap<String,Client>>>;

pub struct Server {
    server: Nickel,
    ad: Ldap,

    config: ::WebConfig,

    clients: Clients,
}

impl Server {
    pub fn new(ad: Ldap, config: &::Config) -> Server {
        let server = Nickel::new();
        let mut s = Server {
            server:server,
            ad:ad,

            config: config.web.clone(),

            clients: Arc::new(Mutex::new(HashMap::new())),
        };

        s.apply_routes();
        s
    }

    fn apply_routes(&mut self) {
        let ad = Arc::new(self.ad.clone());
        let max_age = self.config.session as u64;
        
        // we must make BC happy
        let clients_set_auth = self.clients.clone();
        let clients_get_auth = self.clients.clone();
        
        self.server.get("/", middleware!
                        { |req, res|
                           if Server::is_auth(req, &clients_get_auth, max_age) {
                               return res.redirect("/special")
                           }
                           
                           return res.redirect("/login")
                        });

        self.server.get("/login", middleware!
                        { |_rq, res|
                            let mut data = HashMap::new();
                            data.insert("target","/auth");
                            
                            return res.render("views/login.html", &data)
                        });

        self.server.post("/auth", middleware!
                         { |req, mut res|
                            let form_data = try_with!(res, req.form_body());
                            if let Some(username) = form_data.get("username") {
                                let username = username.to_owned();
                                if let Some(password) = form_data.get("password") {
                                    
                                    if !username.is_empty() &&
                                        !password.is_empty() {
                                            if ad.auth(&username,&password) {
                                                if let Ok(mut clients) = clients_set_auth.lock() {
                                                    let sid = random::<u64>() .to_string();
                                                    let cookie_sid = Cookie::new("sid", sid.clone()).to_string();
                                                    let cookie_username = Cookie::new("username", username.clone()).to_string();
                                                    let client = Client { sid: sid, session: Instant::now() };
                                                    clients.insert(username,
                                                                   client);
                                                    
                                                    res.headers_mut().set_raw("Set-Cookie",
                                                                              vec![cookie_sid.as_bytes().to_vec(),
                                                                                   cookie_username.as_bytes().to_vec()]);
                                                }
                                                
                                                return res.redirect("/")
                                            }
                                        }
                                }
                            }

                            let mut data = HashMap::new();
                            data.insert("target","/login"); //start over
                            return res.render("views/invalid.html", &data)
                         });
    }

    #[allow(dead_code)]
    pub fn get_mut(&mut self) -> &mut Nickel {
        &mut self.server
    }

    pub fn start(self) {
        let address = self.config.ip + ":" + &self.config.port.to_string();
        if let Ok(_s) = self.server.listen(&address) {
            // NOTE: we may detach this at some point!
        }
    }

    fn parse_cookies(raw: &[Vec<u8>]) -> Vec<Cookie> {
        let mut cookies = vec!();
        for cookies_raw in raw.iter() {
            let cookies_str = String::from_utf8(cookies_raw.clone())
                .expect("Non-utf8 encoding encountered");
            for cookie_str in cookies_str.split(';') {
                let s = cookie_str.trim().to_owned();
                if let Ok(cookie) = Cookie::parse(s) {
                    cookies.push(cookie);
                }
                
            }
        }

        cookies
    }

    fn get_cookie<'a>(name: &str, cookies: &'a[Cookie]) -> Option<&'a str> {
        for c in cookies {
            let kv = c.name_value();
            if kv.0 == name {
                return Some(kv.1)
            }
        }

        None
    }

    fn is_auth (req: &Request, cm: &Clients, max_age: u64) -> bool {
        if let Some(cookies) =  req.origin.headers.get_raw("Cookie") {
            let cookies = Server::parse_cookies(cookies);
            if let Some(username) = Server::get_cookie("username", &cookies) {
                if let Some(sid) = Server::get_cookie("sid", &cookies) {
                    if let Ok(cm) = cm.lock() {
                        if let Some(c) = cm.get(username) {
                            if c.sid == sid {
                                if c.session.elapsed() < Duration::from_secs(max_age) {
                                    return true
                                }
                            }
                        }
                    }
                }
            }
        }

        false
    }
}
