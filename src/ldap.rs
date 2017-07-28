extern crate ldap3;

use self::ldap3::LdapConn;

#[derive(Clone)]
pub struct Ldap {
    host: String,
    domain: String,
}

impl Ldap {
    pub fn new (config: &::Config) -> Ldap {
        let mut host = "ldap://".to_owned();
        host.push_str(&config.ldap_server);

        Ldap {
            host:host,
            domain: config.ldap_domain.clone()
        }
        
    }
    
    pub fn auth(&self, username: &str, password: &str) -> bool {
        let username = username.to_owned() + "@" + &self.domain;

        let ldap = LdapConn::new(&self.host).expect("Unable to connect to ldap server");
        let r = ldap.simple_bind(&username, password)
            .expect("Unable to bind to ldap").success();
        r.is_ok()
    }
}
