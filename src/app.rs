use imports::*;
use algos::methods::ClientMethod;
use std::cell::RefCell;

#[derive(Eq, PartialEq)]
pub enum State {
    Initial,
    Error,
    LoggedIn,
    Exit,
}

#[allow(unused)]
pub enum AppEvent {
    Input(Event),
    Online(Online),
    Updates(Updates),
}

pub struct App {
    state: State,
    input: TextField,
    status: String,

    // users
    me: Username,
    peer: Username,

    // app internals
    events: (Sender<AppEvent>, Receiver<AppEvent>),

    // tui internals
    #[allow(unused)] screen: AlternateScreen<io::Stdout>,
    t: RefCell<Terminal<TermionBackend>>,
    size: Rect,
}

impl App {
    pub fn new(me: Username, peer: Username) -> App {
        let screen = AlternateScreen::from(io::stdout());
        let backend = TermionBackend::new().unwrap();
        let mut t = Terminal::new(backend).unwrap();
        t.hide_cursor().unwrap();

        App {
            state: State::Initial,
            input: TextField::new(),
            status: String::new(),

            me,
            peer,

            events: channel(),

            screen,
            t: RefCell::new(t),
            size: Default::default(),
        }
    }

    pub fn error(&mut self, msg: String) {
        self.state = State::Error;
        self.status = msg;
    }

    fn set_up(&mut self) {
        let tx = self.events.0.clone();
        thread::spawn(move || {
            let stdin = io::stdin();
            for event in stdin.events() {
                let event = event.unwrap();
                tx.send(AppEvent::Input(event)).unwrap();
            }
        });
    }

    fn login(&mut self) {
        let method = Login { username: self.me.clone() };
        match method.invoke(&Connection::default()) {
            Ok(answer) => {
                match answer {
                    LoginResult::LoginOk { username } => {
                        self.me = username;
                        self.state = State::LoggedIn;
                        self.status = format!("Logged in as {}", self.me);
                    }
                    LoginResult::LoginErr => {
                        let msg = format!("Login error: username \"{}\" can not be used", &self.me);
                        self.error(msg);
                    }
                }
            }
            Err(e) => {
                self.error(format!("{:?}", e));
            }
        }
    }

    pub fn event_loop(&mut self) -> io::Result<()> {
        self.set_up();
        self.login();

        while self.state != State::Exit {
            self.resize_maybe()?;

            {
                let t = &mut *self.t.borrow_mut();
                self.draw(t)?;
                // drop t
            }

            let event = self.events.1.recv().map_err(|_| io::Error::from(io::ErrorKind::Other))?;
            self.handle_app_event(event);
        }

        Ok(())
    }

    fn resize_maybe(&mut self) -> io::Result<()> {
        let t = &mut *self.t.borrow_mut();

        let size = t.size()?;
        if size != self.size {
            t.resize(size)?;
            self.size = size;
        }
        Ok(())
    }

    fn draw(&self, t: &mut Terminal<TermionBackend>) -> io::Result<()> {
        Group::default()
            .direction(Direction::Vertical)
            .margin(0)
            .sizes(&[Size::Min(0), Size::Fixed(2)]) // status bar at the bottom
            .render(t, &self.size, |t, chunks| {
                Block::default()
                    .borders(border::ALL)
                    .render(t, &chunks[0]);

                StatusBar::default()
                    .message(&self.status)
                    .error(self.state == State::Error)
                    .render(t, &chunks[1]);
            });
        t.draw()?;
        Ok(())
    }

    fn handle_app_event(&mut self, _event: AppEvent) {
        match _event {
            AppEvent::Input(ev) => self.handle_input(ev),
            _ => {}
        }
        self.state = State::Exit;
    }

    fn handle_input(&mut self, event: Event) {}
}

impl Drop for App {
    fn drop(&mut self) {
        let t = &mut *self.t.borrow_mut();

        t.show_cursor().unwrap();
        t.clear().unwrap();
    }
}