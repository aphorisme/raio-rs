pub struct AuthData {
    pub scheme: String,
    pub principal: String,
    pub credentials: String,
}
pub trait AuthMethod {
    fn into_auth_data(self) -> AuthData;
}

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