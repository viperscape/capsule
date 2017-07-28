use nickel::{Nickel, HttpRouter, FormBody};

use std::collections::HashMap;
use std::sync::Arc;
use ldap::Ldap;

pub struct Server {
    server: Nickel,
    ad: Ldap,

    ip: String,
    port: String,
}

impl Server {
    pub fn new(ad: Ldap, config: &::Config) -> Server {
        let server = Nickel::new();
        let mut s = Server {
            server:server,
            ad:ad,

            ip: config.web_ip.clone(),
            port: config.web_port.clone(),
        };

        s.apply_routes();
        s
    }

    fn apply_routes(&mut self) {
        let ad = Arc::new(self.ad.clone());
        
        self.server.get("/", middleware!("Hello World"));

        self.server.get("/login", middleware!
                        { |_rq, res|
                            let mut data = HashMap::new();
                            data.insert("target","/auth");
                            
                            return res.render("views/login.html", &data)
                        });

        self.server.post("/auth", middleware!
                         { |req, res|
                            let form_data = try_with!(res, req.form_body());
                            if let Some(username) = form_data.get("username") {
                                if let Some(password) = form_data.get("password") {
                                    
                                    if !username.is_empty() &&
                                        !password.is_empty() {
                                            if ad.auth(&username,&password) {
                                                let mut data = HashMap::new();
                                                data.insert("authenticated","true");
                                                return res.render("views/auth.html", &data)
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
        let address = self.ip + ":" + &self.port;
        if let Ok(_s) = self.server.listen(&address) {
            // NOTE: we may detach this at some point!
        }
    }
}
