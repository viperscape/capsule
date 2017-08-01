extern crate ldap3;

use self::ldap3::LdapConn;

#[derive(Clone)]
pub struct Ldap {
    host: String,
    domain: String,
}

impl Ldap {
    pub fn new (config: &::Config) -> Ldap {
        let mut host;
        
        if config.domain.secure { host = "ldaps://".to_owned(); }
        else { host = "ldap://".to_owned(); }
        
        host.push_str(&config.domain.server);

        Ldap {
            host:host,
            domain: config.domain.domain.clone()
        }
        
    }
    
    pub fn auth(&self, username: &str, password: &str) -> bool {
        let mut username = username.to_owned();
        if !username.contains("@") {
            username = username + "@" + &self.domain;
        }

        let ldap = LdapConn::new(&self.host).expect("Unable to connect to ldap server");
        let r = ldap.simple_bind(&username, password)
            .expect("Unable to bind to ldap").success();
        r.is_ok()
    }
}
