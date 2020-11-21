/// The general form of authentication data. It Is mainly used by
/// [`AuthMethod`](crate::client::auth::AuthMethod).
pub struct AuthData {
    pub scheme: String,
    pub principal: String,
    pub credentials: String,
}
pub trait AuthMethod {
    fn into_auth_data(self) -> AuthData;
}

/// The basic auth method, which uses a user name and a password.
/// ```
/// # use raio::client::auth::{Basic, AuthMethod};
/// let auth = Basic::new("neo4j", "mastertest");
/// let auth_data = auth.into_auth_data();
///
/// assert_eq!(auth_data.scheme, "basic");
/// assert_eq!(auth_data.principal, "neo4j");
/// assert_eq!(auth_data.credentials, "mastertest");
/// ```
pub struct Basic {
    user: String,
    password: String,
}

impl Basic {
    pub fn new(user: &str, password: &str) -> Self {
        Basic {
            user: String::from(user),
            password: String::from(password),
        }
    }
}

impl AuthMethod for Basic {
    fn into_auth_data(self) -> AuthData {
        AuthData {
            scheme: String::from("basic"),
            principal: self.user,
            credentials: self.password,
        }
    }
}