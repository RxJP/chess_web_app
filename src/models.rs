use rocket::form::FromForm;
use std::collections::HashMap;
use std::sync::Mutex;

pub const SESSION_DB_PATH: &str = "./data/session.db";
pub const USER_DB_PATH: &str = "./data/user.db";

#[derive(Clone, Debug)]
pub struct User {
    pub username: Username,
    pub email: EmailId,
    pub password_hash: String,
    pub rating: u16,
}
pub type SessionId = String;
pub type EmailId = String;
pub type Username = String;

#[derive(Debug)]
pub struct SessionDB {
    map: Mutex<HashMap<SessionId, EmailId>>,
}

impl SessionDB {//todo add a session expiration 2-3months out
    pub fn initialize() -> SessionDB {
        Self {
            map: Mutex::new(HashMap::new())
        }
    }

    pub fn get(&self, id: &SessionId) -> Option<EmailId> {
        match self.map.lock() {
            Ok(map) => {
                return match map.get(id) {
                    None => {None}
                    Some(val) => {Some(val.clone())}
                };
            }
            Err(err) => {
                eprintln!("Error occurred while locking session mutex:\n{}", err);
                panic!("IDK1");
            }
        };
    }

    pub fn contains_session(&self, id: &SessionId) -> bool {
        match self.map.lock() {
            Ok(map) => {return map.contains_key(id);}
            Err(err) => {
                eprintln!("Error occurred while locking session mutex:\n{}", err);
                panic!("IDK2");
            }
        };
    }

    pub fn insert(&self, id: SessionId, user_id: EmailId) -> Option<EmailId> {
        match self.map.lock() {
            Ok(mut map) => {return map.insert(id, user_id);}
            Err(err) => {
                eprintln!("Error occurred while locking session mutex:\n{}", err);
                panic!("IDK3");
            }
        };
    }

    pub fn remove_session(&self, id: &SessionId) -> Option<EmailId> {
        match self.map.lock() {
            Ok(mut map) => {return map.remove(id);}
            Err(err) => {
                eprintln!("Error occurred while locking session mutex:\n{}", err);
                panic!("IDK3.1");
            }
        };
    }
}


#[derive(Debug)]
pub struct UserDB {
    map: Mutex<HashMap<EmailId, User>>,
    username_map: Mutex<HashMap<Username, EmailId>>
}

impl UserDB {
    pub fn initialize() -> UserDB {
        Self {
            map: Mutex::new(HashMap::new()),
            username_map: Mutex::new(HashMap::new()),
        }
    }

    pub fn get_email_from_username(&self, username: &Username) -> Option<EmailId> {
        match self.username_map.lock() {
            Ok(map) => {
                return match map.get(username) {
                    None => {None}
                    Some(val) => {Some(val.clone())}
                };
            }
            Err(err) => {
                eprintln!("Error occurred while locking user mutex:\n{}", err);
                panic!("IDK4.1");
            }
        };
    }

    pub fn get_user_from_username(&self, username: &Username) -> Option<User> {
        let email = match self.username_map.lock() {
            Ok(map) => {
                match map.get(username) {
                    None => { return None; }
                    Some(val) => { val.clone() }
                }
            }
            Err(err) => {
                eprintln!("Error occurred while locking user mutex:\n{}", err);
                panic!("IDK4.1");
            }
        };

        self.get_user_from_email(&email)
    }

    pub fn contains_username(&self, username: &Username) -> bool {
        match self.username_map.lock() {
            Ok(map) => {return map.contains_key(username);}
            Err(err) => {
                eprintln!("Error occurred while locking user user mutex:\n{}", err);
                panic!("IDK5");
            }
        };
    }

    pub fn get_user_from_email(&self, email_id: &EmailId) -> Option<User> {
        match self.map.lock() {
            Ok(map) => {
                return match map.get(email_id) {
                    None => {None}
                    Some(val) => {Some(val.clone())}
                };
            }
            Err(err) => {
                eprintln!("Error occurred while locking user mutex:\n{}", err);
                panic!("IDK4.2");
            }
        };
    }

    pub fn contains_email(&self, id: &EmailId) -> bool {
        match self.map.lock() {
            Ok(map) => {return map.contains_key(id);}
            Err(err) => {
                eprintln!("Error occurred while locking user user mutex:\n{}", err);
                panic!("IDK5");
            }
        };
    }

    pub fn insert_user(&self, email_id: EmailId, user: User) -> Option<User> {
        let username = user.username.clone();
        let val = match self.map.lock() {
            Ok(mut map) => {map.insert(email_id.clone(), user)}
            Err(err) => {
                eprintln!("Error occurred while locking user mutex:\n{}", err);
                panic!("IDK6");
            }
        };
        let _ = match self.username_map.lock() {
            Ok(mut map) => {map.insert(username, email_id)}
            Err(err) => {
                eprintln!("Error occurred while locking user mutex:\n{}", err);
                panic!("IDK6");
            }
        };

        return val;
    }
}

#[derive(FromForm)]
pub struct LoginForm {
    pub id: String,
    pub password: String,
    pub login_type: String,
}

#[derive(FromForm)]
pub struct RegistrationForm {
    pub email_id: String,
    pub username: String,
    pub password: String,
}