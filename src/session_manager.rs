use hmac::{Hmac, NewMac};
use jwt::SignWithKey;
use jwt::VerifyWithKey;
use ring::rand::{SecureRandom, SystemRandom};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

// TODO: only log errors in production mode, dont send them to client

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Session {
    pub id: String,
    pub valid_until: u64,
}

pub struct SessionManager {
    rand: SystemRandom,
    lifetime: u64,
    sessions: Mutex<HashMap<String, Session>>,
    key: Hmac<Sha256>,
}

impl SessionManager {
    pub fn new(lifetime: u64) -> Result<SessionManager, String> {
        let mut key: [u8; 32] = [0; 32];
        let rand = SystemRandom::new();
        if let Err(e) = rand.fill(&mut key) {
            return Err(format!("Could not get random key: {}", e));
        }

        let key = Hmac::new_varkey(&key)
            .map_err(|e| format!("Could not create HMAC: {}", e))?;

        return Ok(SessionManager {
            rand: rand,
            lifetime: lifetime,
            sessions: Mutex::new(HashMap::new()),
            key: key,
        });
    }

    pub fn register(&self) -> Result<String, String> {
        let mut raw_id: [u8; 30] = [0; 30];
        if let Err(e) = self.rand.fill(&mut raw_id) {
            return Err(format!("Could not generate session ID: {}", e));
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| format!("Get system time failed: {}", e))?
            .as_secs();
        let valid_until = match now.checked_add(self.lifetime) {
            Some(x) => x,
            None => return Err("Could not generate timestamp".into()),
        };

        let session = Session {
            id: base64::encode(&raw_id),
            valid_until: valid_until,
        };

        // XXX: needs session.clone() because sign_with_key consumes object
        let token: String = session
            .clone()
            .sign_with_key(&self.key)
            .map_err(|e| format!("Could not sign new session: {}", e))?;

        match self.sessions.lock() {
            Ok(mut x) => {
                x.retain(|_k, v| v.valid_until > now);
                x.insert(session.id.clone(), session);
            }
            Err(e) => return Err(format!("Locking sessions failed: {}", e)),
        }
        return Ok(token);
    }

    fn verify_token(&self, token: &String) -> Result<Session, String> {
        return token.verify_with_key(&self.key).map_err(|e| {
            if cfg!(debug_assertions) {
                return format!("Could not verify session: {}", e);
            } else {
                return "Unauthorized!".into();
            }
        });
    }

    pub fn verify(&self, token: &String) -> Result<(), String> {
        let requested_session = self.verify_token(token)?;

        match self.sessions.lock() {
            Ok(mut x) => match x.get_mut(&requested_session.id) {
                Some(mut session) => {
                    let now = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .map_err(|e| format!("Get system time failed: {}", e))?
                        .as_secs();
                    if session.valid_until < now {
                        x.remove(&requested_session.id);
                        return Err("Session expired!".into());
                    }
                    match now.checked_add(self.lifetime) {
                        Some(x) => session.valid_until = x,
                        None => {
                            return Err("Could not generate timestamp".into())
                        }
                    };
                }
                None => {
                    if cfg!(debug_assertions) {
                        return Err("Could not find session".into());
                    } else {
                        return Err("Unauthorized!".into());
                    }
                }
            },
            Err(e) => return Err(format!("Locking sessions failed: {}", e)),
        }
        return Ok(());
    }

    pub fn destroy(&self, token: &String) -> Result<(), String> {
        let requested_session = self.verify_token(token)?;

        match self.sessions.lock() {
            Ok(mut x) => {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map_err(|e| format!("Get system time failed: {}", e))?
                    .as_secs();
                x.retain(|_k, v| v.valid_until > now);

                if let None = x.remove(&requested_session.id) {
                    if cfg!(debug_assertions) {
                        return Err("Could not find session".into());
                    } else {
                        return Err("Unauthorized!".into());
                    }
                }
            }
            Err(e) => return Err(format!("Locking sessions failed: {}", e)),
        }
        return Ok(());
    }
}
