// network
extern crate reqwest;
// serde
extern crate serde;
extern crate serde_json;
extern crate base64;
// gui
extern crate tui;
extern crate termion;
// algos
extern crate algos;
extern crate rand;

mod app;
mod error;
mod imports;
mod connection;
mod gui;
mod logo;
mod mode;

use imports::*;

fn usage() -> ! {
    let name = env::args().next().unwrap();
    println!("usage: {} <username> <peer username>", name);
    exit(1);
}

fn main() {
    let mut args = env::args().skip(1);
    let me = match args.next() {
        Some(ok) => ok,
        None => usage(),
    };
    let peer = match args.next() {
        Some(ok) => ok,
        None => usage(),
    };

    let mut app = app::App::new(me, peer);

    let result = catch_unwind(AssertUnwindSafe(|| {
        app.event_loop().unwrap();
    }));

    // return screen to normal
    drop(app);

    if let Err(e) = result {
        match e.downcast::<&str>() {
            Ok(s) => println!("{}", *s),
            Err(_) => println!("Unknown error"),
        }
        exit(1);
    }
}
