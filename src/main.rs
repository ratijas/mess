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
//!     * `FileUpdate from:Username to:Username coding:string compression:string file:FileMeta file_id:FileId = Update`
//!
//! - `FileMeta`:
//!     * `FileMeta name:string size:int mime:string = FileMeta`
//!
//! - `FileId`
//!     * `FileId file_id:int = FileId`
//!
//!
//! server methods:
//! - `login username:string = LoginResult`
//! - `online username:string = Online`
//! - `getUpdates username:string = Updates`
//! - `sendText from:Username to:Username coding:string compression:string text:string = Bool`
//! - `sendFile = FileId`
//! - `uploadFile from:Username to:Username coding:string compression:string file:FileMeta file_id:FileId bytes:bytes = Bool`
//! - `downloadFile file_id:FileId = bytes`
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
use std::collections::HashSet;
use std::collections::VecDeque;

use std::io::Read;

use types::*;
use methods::*;

#[derive(Debug)]
pub struct App {
    pub users: HashMap<Username, User>,
    pub pending: HashSet<FileId>,
    pub files: HashMap<FileId, File>,
    pub last_id: i32,
}

#[derive(Debug)]
pub struct User {
    pub username: Username,
    pub inbox: VecDeque<Update>,
}

#[derive(Debug)]
pub struct File {
    pub meta: FileMeta,
    pub content: String,
}

pub struct JsonKey;

impl Key for JsonKey { type Value = serde_json::Value; }

impl Key for App { type Value = App; }

impl App {
    pub fn new() -> Self {
        App {
            users: HashMap::new(),
            pending: HashSet::new(),
            files: HashMap::new(),
            last_id: 0,
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
    router.post("/", default, "default");

    router.post("/login", handle_method::<Login>, "login");
    router.post("/online", handle_method::<GetOnline>, "online");
    router.post("/getUpdates", handle_method::<GetUpdates>, "getUpdates");
    router.post("/sendText", handle_method::<SendText>, "sendText");
    router.post("/sendFile", handle_method::<SendFile>, "sendFile");
    router.post("/uploadFile", handle_method::<UploadFile>, "uploadFile");
    router.post("/downloadFile", handle_method::<DownloadFile>, "downloadFile");

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
        FileUpdate { from:Username, to:Username, coding:String, compression:String, file:FileMeta , file_id:FileId },
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
        SentFile { file_id: i32 }
    }

    #[derive(Clone, Debug)]
    #[derive(Serialize, Deserialize)]
    #[serde(untagged)]
    pub enum FileMeta {
        FileMeta { name:String, size:i32, mime:String }
    }

    #[derive(Clone, Debug, PartialEq, Eq, Hash)]
    #[derive(Serialize, Deserialize)]
    #[serde(untagged)]
    pub enum FileId {
        FileId { file_id: i32 }
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
    pub struct GetOnline {}

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
    pub struct SendText {
        pub from: Username,
        pub to: Username,
        pub coding: String,
        pub compression: String,
        pub text: String,
    }

    impl Method for SendText {
        type Answer = bool;

        fn invoke(self, app: &mut App) -> bool {
            if self.to == self.from || !app.users.contains_key(&self.to) { return false };
            app.users.get_mut(&self.to).unwrap().inbox.push_back(Update::TextUpdate
                { from: self.from, to: self.to, coding: self.coding, compression: self.compression, text: self.text });
            true
        }
    }

    #[derive(Clone, Debug)]
    #[derive(Serialize, Deserialize)]
    pub struct SendFile {}

    impl Method for SendFile {
        type Answer = FileId;

        fn invoke(self, app: &mut App) -> FileId {
            let file_id = FileId::FileId { file_id: app.last_id };
            app.last_id += 1;
            app.pending.insert(file_id.clone());
            file_id
        }
    }

    #[derive(Clone, Debug)]
    #[derive(Serialize, Deserialize)]
    pub struct UploadFile {
        pub from: Username,
        pub to: Username,
        pub coding: String,
        pub compression: String,
        pub file: FileMeta,
        pub file_id: i32,
        pub bytes: String,
    }

    impl Method for UploadFile {
        type Answer = bool;

        fn invoke(self, app: &mut App) -> bool {
            if self.to == self.from || !app.users.contains_key(&self.to) { return false };
            if !app.pending.contains(&FileId::FileId { file_id: self.file_id }) ||
                app.files.contains_key(&FileId::FileId { file_id: self.file_id }) { return false };
            app.files.insert(FileId::FileId { file_id: self.file_id }, File { meta: self.file.clone() , content: self.bytes });
            app.pending.remove(&FileId::FileId { file_id: self.file_id });
            app.users.get_mut(&self.to).unwrap().inbox.push_back(Update::FileUpdate
                { from: self.from, to: self.to, coding: self.coding, compression: self.compression, file: self.file, file_id: FileId::FileId { file_id: self.file_id } });
            true
        }
    }

    #[derive(Clone, Debug)]
    #[derive(Serialize, Deserialize)]
    pub struct DownloadFile {
        pub file_id: i32,
    }

    impl Method for DownloadFile {
        type Answer = String;

        fn invoke(self, app: &mut App) -> String {
            app.files.get(&FileId::FileId { file_id: self.file_id}).unwrap().content.clone()
        }
    }
}
