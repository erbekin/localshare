//! This module encapsulates authentication part
use crate::config::SESSION_COOKIE_NAME;
use anyhow::Context;
use rocket::{
    Request,
    http::{Cookie, Status},
    request::{FromRequest, Outcome},
};
use std::{collections::HashSet, hash::Hash, str::FromStr};
use tokio::sync::Mutex;
use uuid::Uuid;

/// Unique ID representing a session
#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub struct SessionId(Uuid);

impl SessionId {
    pub fn generate() -> Self {
        Self(Uuid::new_v4())
    }
}

impl FromStr for SessionId {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let uuid = s
            .parse::<Uuid>()
            .context("failed to convert string to uuid")?;
        Ok(Self(uuid))
    }
}

impl From<SessionId> for Cookie<'static> {
    fn from(value: SessionId) -> Self {
        let mut cookie = Self::new(SESSION_COOKIE_NAME, value.0.to_string());
        cookie.set_path("/");
        cookie.set_http_only(true); // dont allow js
        cookie.set_same_site(rocket::http::SameSite::Strict);
        cookie
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for SessionId {
    type Error = ();
    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let session_id = request
            .cookies()
            .get_private(SESSION_COOKIE_NAME)
            .map(|c| c.value().to_string());
        match session_id {
            Some(s) => {
                let session_storage = request
                    .rocket()
                    .state::<Mutex<SessionStorage>>()
                    .expect("rocket manages session storage");
                let session_id: SessionId = match s.parse() {
                    Ok(id) => id,
                    Err(_) => return Outcome::Error((Status::BadRequest, ())),
                };
                let contains_it = { session_storage.lock().await.contains(&session_id) };
                if contains_it {
                    Outcome::Success(session_id)
                } else {
                    Outcome::Error((Status::Unauthorized, ()))
                }
            }
            None => Outcome::Error((Status::Unauthorized, ())),
        }
    }
}

/// Stores the sessions
pub struct SessionStorage {
    sessions: HashSet<SessionId>,
}

impl SessionStorage {
    pub fn new() -> Self {
        Self {
            sessions: HashSet::new(),
        }
    }
    pub fn contains(&self, session_id: &SessionId) -> bool {
        self.sessions.contains(session_id)
    }
    pub fn insert(&mut self, session_id: SessionId) {
        self.sessions.insert(session_id);
    }
    pub fn remove(&mut self, session_id: &SessionId) {
        self.sessions.remove(session_id);
    }
}
