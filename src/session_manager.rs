use hmac::{Hmac, NewMac};
use jwt::SignWithKey;
use jwt::VerifyWithKey;
use ring::rand::{SecureRandom, SystemRandom};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use slog::{error, Logger};
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Session {
    pub id: String,
    pub valid_until: u64,
}

pub struct AuthError {
    message: Option<String>,
    user_message: Option<String>,
}

impl AuthError {
    pub fn new(message: String) -> AuthError {
        return AuthError {
            message: Some(message),
            user_message: None,
        };
    }

    pub fn new_custom(message: String, user_message: String) -> AuthError {
        return AuthError {
            message: Some(message),
            user_message: Some(user_message),
        };
    }

    pub fn new_user(message: String) -> AuthError {
        return AuthError {
            message: Some(message.clone()),
            user_message: Some(message),
        };
    }

    pub fn to_string(self, logger: &Logger) -> String {
        if let Some(message) = self.message {
            error!(logger, "{}", message);
        }
        return self.user_message.unwrap_or("Internal server error!".into());
    }
}

macro_rules! auth_error(
    ($format:expr) => {
        AuthError::new($format.to_string())
    };
    ($format:expr, $($args:tt)+) => {
        AuthError::new(format!($format, $($args)+))
    };
);

pub struct SessionManager {
    rand: SystemRandom,
    lifetime: u64,
    // TODO: trace content of sessions
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

    pub fn register(&self) -> Result<String, AuthError> {
        let mut raw_id: [u8; 30] = [0; 30];
        if let Err(e) = self.rand.fill(&mut raw_id) {
            return Err(auth_error!("Could not generate session ID: {}", e));
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| auth_error!("Get system time failed: {}", e))?
            .as_secs();
        let valid_until = match now.checked_add(self.lifetime) {
            Some(x) => x,
            None => return Err(auth_error!("Could not generate timestamp")),
        };

        let session = Session {
            id: base64::encode(&raw_id),
            valid_until: valid_until,
        };

        // XXX: needs session.clone() because sign_with_key consumes object
        let token: String = session
            .clone()
            .sign_with_key(&self.key)
            .map_err(|e| auth_error!("Could not sign new session: {}", e))?;

        match self.sessions.lock() {
            Ok(mut x) => {
                x.retain(|_k, v| v.valid_until > now);
                x.insert(session.id.clone(), session);
            }
            Err(e) => {
                return Err(auth_error!("Locking sessions failed: {}", e))
            }
        }
        return Ok(token);
    }

    fn verify_token(&self, token: &String) -> Result<Session, AuthError> {
        return token.verify_with_key(&self.key).map_err(|e| {
            AuthError::new_custom(
                format!("Could not verify session: {}", e),
                "Unauthorized!".into(),
            )
        });
    }

    pub fn verify(&self, token: &String) -> Result<(), AuthError> {
        let requested_session = self.verify_token(token)?;

        match self.sessions.lock() {
            Ok(mut x) => match x.get_mut(&requested_session.id) {
                Some(mut session) => {
                    let now = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .map_err(|e| {
                            auth_error!("Get system time failed: {}", e)
                        })?
                        .as_secs();
                    if session.valid_until < now {
                        x.remove(&requested_session.id);
                        return Err(AuthError::new_user(
                            "Session expired!".into(),
                        ));
                    }
                    match now.checked_add(self.lifetime) {
                        Some(x) => session.valid_until = x,
                        None => {
                            return Err(auth_error!(
                                "Could not generate timestamp"
                            ))
                        }
                    };
                }
                None => {
                    return Err(AuthError::new_custom(
                        "Could not find session".into(),
                        "Unauthorized!".into(),
                    ))
                }
            },
            Err(e) => {
                return Err(auth_error!("Locking sessions failed: {}", e))
            }
        }
        return Ok(());
    }

    pub fn destroy(&self, token: &String) -> Result<(), AuthError> {
        let requested_session = self.verify_token(token)?;

        match self.sessions.lock() {
            Ok(mut x) => {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map_err(|e| auth_error!("Get system time failed: {}", e))?
                    .as_secs();
                x.retain(|_k, v| v.valid_until > now);

                if let None = x.remove(&requested_session.id) {
                    return Err(AuthError::new_custom(
                        "Could not find session".into(),
                        "Unauthorized!".into(),
                    ));
                }
            }
            Err(e) => {
                return Err(auth_error!("Locking sessions failed: {}", e))
            }
        }
        return Ok(());
    }
}
