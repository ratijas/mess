extern crate iron;
extern crate router;
extern crate persistent;
extern crate error as e;
extern crate typemap;

extern crate serde_json;
extern crate serde;

#[macro_use]
extern crate log;
extern crate fern;
extern crate chrono;

extern crate algos;

use iron::prelude::*;
use iron::{headers, status};
use iron::modifiers::Header;

use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::io::Read;

use typemap::Key;
use persistent::State;

use algos::methods::*;
use algos::types::*;

pub mod server;

#[derive(Debug)]
pub struct App {
    pub users: HashMap<Username, User>,
    /// set of requested but not yet uploaded files
    pub pending: HashSet<FileId>,
    /// set of uploaded files waiting for receiver to fetch them
    pub files: HashMap<FileId, File>,
    /// last used file id
    pub last_id: i64,
}

#[derive(Debug)]
pub struct User {
    pub username: Username,
    pub inbox: VecDeque<Update>,
}

#[derive(Debug)]
pub struct File {
    pub meta: FileMeta,
    pub payload: Data,
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

fn app_handler() -> Chain {
    let mut r = router::Router::new();
    r.post("/", default, "default");

    fn route<M: 'static + ServerMethod<App>>(r: &mut router::Router) {
        r.get(&format!("/{}", M::endpoint()), handle_method::<M>, M::endpoint());
        r.post(&format!("/{}", M::endpoint()), handle_method::<M>, M::endpoint());
    }

    route::<Login>(&mut r);
    route::<GetOnline>(&mut r);
    route::<GetUpdates>(&mut r);
    route::<SendText>(&mut r);
    route::<SendFile>(&mut r);
    route::<UploadFile>(&mut r);
    route::<DownloadFile>(&mut r);

    let mut chain = Chain::new(r);
    chain.link(State::<App>::both(App::new()));

    // every response is a json
    chain.link_after(|_: &mut Request, res: Response| -> IronResult<Response> {
        let res = res.set(Header(headers::ContentType::json()));
        Ok(res)
    });

    chain
}

fn handle_method<M: ServerMethod<App>>(req: &mut Request) -> IronResult<Response> {
    let method: M = match parse_method(req) {
        Ok(m) => m,
        Err(body) => {
            error!("body is not a json! {:?} {:?}", req, body);
            return bad_request();
        }
    };

    let answer: M::Answer = {
        let lock = req.get::<State<App>>().unwrap();
        let mut app = lock.write().unwrap();
        let res = method.handle(&mut app);

        debug!("app: {:?}", &*app);

        res
    };

    Ok(Response::with((status::Ok,
                       serde_json::to_string(&GeneralAnswer::Ok(answer)).unwrap())))
}

fn parse_method<M: Method>(req: &mut Request) -> Result<M, Vec<u8>> {
    let mut body = Vec::new();
    if let Ok(_) = req.body.read_to_end(&mut body) {
        if body.is_empty() {
            body.extend(b"{}");
        }

        serde_json::from_slice(&body).map_err(|_| body)
    } else {
        Err(body)
    }
}

fn bad_request() -> IronResult<Response> {
    let answer: GeneralAnswer<()> = GeneralAnswer::Err("bad request".into());
    Ok(Response::with((
        status::Ok,
        serde_json::to_string(&answer).unwrap(),
    )))
}

fn default(_: &mut Request) -> IronResult<Response> {
    println!("imitating 5 seconds delay");
    ::std::thread::sleep(::std::time::Duration::from_secs(5));
    bad_request()
}

pub fn setup_log(level: log::LogLevelFilter) {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(level)
        .chain(std::io::stdout())
        .apply()
        .unwrap();
}

fn main() {
    setup_log(log::LogLevelFilter::Info);

    let handler = app_handler();
    Iron::new(handler).http("0.0.0.0:3000").unwrap();
}
