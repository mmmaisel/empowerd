use hmac::{Hmac, NewMac};
use jwt::SignWithKey;
use jwt::VerifyWithKey;
use ring::rand::{SecureRandom, SystemRandom};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::collections::BTreeMap;
use std::sync::RwLock;

// TODO: only log errors in production mode, dont send them to client

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Session {
    pub id: String,
    pub valid_until: u64,
}

pub struct SessionManager {
    rand: SystemRandom,
    sessions: RwLock<BTreeMap<String, Session>>,
    key: Hmac<Sha256>,
}

impl SessionManager {
    pub fn new() -> Result<SessionManager, String> {
        let mut key: [u8; 32] = [0; 32];
        let rand = SystemRandom::new();
        if let Err(e) = rand.fill(&mut key) {
            return Err(format!("Could not get random key: {}", e));
        }

        let key = Hmac::new_varkey(&key)
            .map_err(|e| format!("Could not create HMAC: {}", e))?;

        return Ok(SessionManager {
            rand: rand,
            sessions: RwLock::new(BTreeMap::new()),
            key: key,
        });
    }

    pub fn register(&self) -> Result<String, String> {
        let mut raw_id: [u8; 30] = [0; 30];
        if let Err(e) = self.rand.fill(&mut raw_id) {
            return Err(format!("Could not generate session ID: {}", e));
        }

        let session = Session {
            id: base64::encode(&raw_id),
            valid_until: 0, // TODO: implement
        };

        // XXX: needs session.clone() because sign_with_key consumes object
        let token: String = session
            .clone()
            .sign_with_key(&self.key)
            .map_err(|e| format!("Could not sign new session: {}", e))?;

        // TODO: dont unwrap
        self.sessions
            .write()
            .unwrap()
            .insert(session.id.clone(), session);
        // TODO: wipe timeouted sessions
        return Ok(token);
    }

    fn verify_token(&self, token: &String) -> Result<Session, String> {
        return match token.verify_with_key(&self.key) {
            Ok(x) => x,
            Err(e) => {
                if cfg!(debug_assertions) {
                    return Err(format!("Could not verify session: {}", e));
                } else {
                    return Err("Unauthorized!".into());
                }
            }
        };
    }

    pub fn verify(&self, token: &String) -> Result<(), String> {
        let requested_session = self.verify_token(token)?;

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
        let requested_session = self.verify_token(token)?;

        match self.sessions.write().unwrap().remove(&requested_session.id) {
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
        // TODO: wipe timeouted sessions
        return Ok(());
    }
}
