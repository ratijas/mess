//!
//! server constructors, types:
//! - `int = i32`
//! - `Bool`
//!     * `True = Bool`
//!     * `False = Bool`
//! - `bytes`
//!     * b64 encoded string
//!
//! - `Vector<T>`
//!
//! - `User`
//!     * `user username:string = User`
//!
//! - `LoginResult`
//!     * `LoginOk username:string = LoginResult`
//!     * `LoginErr = LoginResult`
//!
//! - `Online`
//!     * `online users:Vector<User> = Online`
//!
//! - `Updates`
//!     * `Updates updates:Vector<Update> = Updates`
//!
//! - `Update`
//!     * `TextUpdate from:Username to:Username coding:string compression:string text:string = Update`
//!     * `FileUpdate from:Username to:Username coding:string compression:string file:string = Update`
//!
//! - `SentUpdate`
//!     * `SentText = SentUpdate`
//!     * `SentFile file_id:int = SentUpdate`
//!
//! - `Payload`
//!     * `Payload file_id:int data:bytes = Payload`
//!
//!
//! server methods:
//! - `login username:string = LoginResult`
//! - `online username:string = Online`
//! - `getUpdates username:string = Updates`
//! - `send update:Update = SentUpdate`
//! - `uploadFile file_id:int bytes:bytes = Bool`
//!
//!
#[allow(unused)]
extern crate iron;
extern crate router;
extern crate persistent;

extern crate typemap;

use typemap::Key;
use persistent::State;

extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;

use serde::Serialize;

use iron::prelude::*;
use iron::status;

use std::collections::HashMap;
use std::collections::VecDeque;

use std::io::Read;

use types::*;
use methods::*;

#[derive(Debug)]
pub struct App {
    pub users: HashMap<Username, User>,
}

#[derive(Debug)]
pub struct User {
    pub username: Username,
    pub inbox: VecDeque<Update>,
}

pub struct JsonKey;

impl Key for JsonKey { type Value = serde_json::Value; }

impl Key for App { type Value = App; }

impl App {
    pub fn new() -> Self {
        App {
            users: HashMap::new()
        }
    }

    pub fn get_or_new_user(&mut self, username: Username) -> &mut User {
        self.users
            .entry(username.clone())
            .or_insert_with(|| User::new(username))
    }
}

impl User {
    pub fn new(username: Username) -> Self {
        User {
            username,
            inbox: VecDeque::new(),
        }
    }

    pub fn validate_username(username: &str) -> bool {
        username.len() >= 3
            && username.chars().next().unwrap().is_alphabetic()
            && username.chars().all(|ch| ch.is_alphanumeric())
    }
}

fn main() {
    let mut router = router::Router::new();
    router.get("/", default, "default");

    router.get("/login", handle_method::<Login>, "login");
    router.get("/online", handle_method::<GetOnline>, "online");
    router.get("/getUpdates", handle_method::<GetUpdates>, "getUpdates");

    let mut chain = Chain::new(router);
    chain.link(State::<App>::both(App::new()));

    // every query is json
    chain.link_before(|req: &mut Request| -> IronResult<()> {
        let mut body = Vec::new();
        if let Ok(_) = req.body.read_to_end(&mut body) {
            if let Ok(json) = serde_json::from_slice(&body) {
                req.extensions.insert::<JsonKey>(json);
            }
        }
        Ok(())
    });

    Iron::new(chain).http("0.0.0.0:3000").unwrap();
}

fn bad_request() -> IronResult<Response> {
    Ok(Response::with((
        status::Ok,
        err_description("bad request").to_string()
    )))
}

fn ok_result<T: Serialize>(result: T) -> serde_json::Value {
    json!({
        "ok": true,
        "result": result,
    })
}

fn err_description<T: Serialize>(err: T) -> serde_json::Value {
    json!({
        "ok": false,
        "description": err,
    })
}

fn default(_: &mut Request) -> IronResult<Response> {
    println!("imitating 5 seconds delay");
    ::std::thread::sleep(::std::time::Duration::from_secs(5));
    Ok(Response::with((status::Ok, json!({
        "ok": false
    }).to_string())))
}

fn handle_method<M: Method>(req: &mut Request) -> IronResult<Response> {
    let m: M = match req.extensions
                        .remove::<JsonKey>()
                        .map(serde_json::from_value)
                        .and_then(Result::ok) {
        None => return bad_request(),
        Some(m) => m,
    };

    let answer: M::Answer = {
        let lock = req.get::<State<App>>().unwrap();
        let mut app = lock.write().unwrap();

        println!("app: {:?}", &*app);

        m.invoke(&mut app)
    };

    let res = ok_result(answer);

    Ok(Response::with((status::Ok, serde_json::to_string(&res).unwrap())))
}

pub mod types {
    use std::collections::VecDeque;
    pub type Username = String;

    #[derive(Clone, Debug)]
    #[derive(Serialize, Deserialize)]
    #[serde(untagged)]
    pub enum LoginResult {
        LoginOk { username: String },
        LoginErr,
    }

    #[derive(Clone, Debug)]
    #[derive(Serialize, Deserialize)]
    #[serde(untagged)]
    pub enum Online {
        Online { users: Vec<Username> }
    }

    #[derive(Clone, Debug)]
    #[derive(Serialize, Deserialize)]
    #[serde(untagged)]
    pub enum Update {
        TextUpdate { from:Username, to:Username, coding:String, compression:String, text:String },
        FileUpdate { from:Username, to:Username, coding:String, compression:String, file:String },
    }

    #[derive(Clone, Debug)]
    #[derive(Serialize, Deserialize)]
    #[serde(untagged)]
    pub enum Updates {
        Updates { updates: VecDeque<Update> }
    }

    #[derive(Clone, Debug)]
    #[derive(Serialize, Deserialize)]
    #[serde(untagged)]
    pub enum SentUpdate {
        SentText,
        SentFile { file_id:i32 }
    }

    #[derive(Clone, Debug)]
    #[derive(Serialize, Deserialize)]
    #[serde(untagged)]
    pub enum Payload {
        Payload { file_id:i32, data:String }
    }
}

pub mod methods {
    use super::*;
    use serde::de::DeserializeOwned;

    pub trait Method: DeserializeOwned {
        type Answer: Serialize;
        fn invoke(self, app: &mut App) -> Self::Answer;
    }

    #[derive(Clone, Debug)]
    #[derive(Serialize, Deserialize)]
    pub struct Login {
        pub username: Username,
    }

    impl Method for Login {
        type Answer = LoginResult;

        fn invoke(self, app: &mut App) -> LoginResult {
            if !User::validate_username(&self.username) {
                return LoginResult::LoginErr;
            }

            let user = app.get_or_new_user(self.username);

            LoginResult::LoginOk { username: user.username.clone() }
        }
    }

    #[derive(Clone, Debug)]
    #[derive(Serialize, Deserialize)]
    pub struct GetOnline{}

    impl Method for GetOnline {
        type Answer = Online;

        fn invoke(self, app: &mut App) -> Online {
            Online::Online { users: app.users.keys().cloned().collect() }
        }
    }

    #[derive(Clone, Debug)]
    #[derive(Serialize, Deserialize)]
    pub struct GetUpdates {
        pub username: Username,
    }

    impl Method for GetUpdates {
        type Answer = Updates;

        fn invoke(self, app: &mut App) -> Updates {
            let result = Updates::Updates { updates: app.users.get(&self.username).unwrap().inbox.clone() };
            app.users.get_mut(&self.username).unwrap().inbox.clear();
            result
        }
    }

    #[derive(Clone, Debug)]
    #[derive(Serialize, Deserialize)]
    pub struct Send {
        pub update: Update,
    }

    impl Method for Send {
        type Answer = SentUpdate;

        fn invoke(self, app: &mut App) -> SentUpdate {
            match &self.update {
                &Update::TextUpdate {ref to, ..} => {
                    app.users.get_mut(to).unwrap().inbox.push_back(self.update.clone());
                    SentUpdate::SentText
                }
                &Update::FileUpdate {ref to, ..} => {
                    app.users.get_mut(to).unwrap().inbox.push_back(self.update.clone());
                    SentUpdate::SentFile { file_id: 1 }
                }
            }
            /*
                pub enum SentUpdate {
                    SentText,
                    SentFile { file_id:i32 }
                }
            //   * `TextUpdate from:Username to:Username coding:string compression:string text:string = Update`
            //   * `FileUpdate from:Username to:Username coding:string compression:string file:string = Update`*/
        }
    }

    #[derive(Clone, Debug)]
    #[derive(Serialize, Deserialize)]
    pub struct UploadFile {
        pub file_id:i32,
        pub bytes:String,
    }

    impl Method for UploadFile {
        type Answer = bool;

        fn invoke(self, app: &mut App) -> bool {
            true
        }
    }
}
