use hmac::{Hmac, NewMac};
use jwt::SignWithKey;
use jwt::VerifyWithKey;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::collections::BTreeMap;
use std::sync::RwLock;

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Session {
    pub id: String,
    pub valid_until: u64,
}

pub struct SessionManager {
    sessions: RwLock<BTreeMap<String, Session>>,
    key: Hmac<Sha256>,
}

impl SessionManager {
    pub fn new() -> Result<SessionManager, String> {
        // TODO: generate random key, check entropy before
        let key = match Hmac::new_varkey(b"not-secret") {
            Ok(x) => x,
            Err(e) => return Err(format!("Could not create HMAC: {}", e)),
        };

        return Ok(SessionManager {
            sessions: RwLock::new(BTreeMap::new()),
            key: key,
        });
    }

    pub fn register(&self) -> Result<String, String> {
        let session = Session {
            id: "1234".into(), // TODO: generate random id
            valid_until: 0,    // TODO: implement
        };

        // XXX: needs session.clone() because sign_with_key consumes object
        let token: String = match session.clone().sign_with_key(&self.key) {
            Ok(x) => x,
            Err(e) => return Err(format!("Could not sign new session: {}", e)),
        };

        // TODO: dont unwrap
        self.sessions
            .write()
            .unwrap()
            .insert(session.id.clone(), session);
        // TODO: wipe timeouted sessions
        return Ok(token);
    }

    pub fn verify(&self, token: &String) -> Result<(), String> {
        let requested_session: Session = match token.verify_with_key(&self.key)
        {
            Ok(x) => x,
            Err(e) => {
                if cfg!(debug_assertions) {
                    return Err(format!("Could not verify session: {}", e));
                } else {
                    return Err("Unauthorized!".into());
                }
            }
        };

        // TODO: dont unwrap
        let session =
            match self.sessions.read().unwrap().get(&requested_session.id) {
                Some(x) => x,
                None => {
                    if cfg!(debug_assertions) {
                        return Err("Could not find session".into());
                    } else {
                        return Err("Unauthorized!".into());
                    }
                }
            };

        // TODO: check timeout

        return Ok(());
    }

    pub fn destroy(&self, token: &String) -> Result<(), String> {
        // TODO: wipe timeout
        return Err("Not implemented yet".into());
    }
}
