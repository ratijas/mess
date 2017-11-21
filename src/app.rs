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

    SentText(SendText),
    SentFile(UploadFile),
    SendFailed { reason: String },

    Log { message: String, error: bool },
}

pub struct App {
    state: State,
    /// user can't send new messages until he finishes with current.
    sending: bool,

    input: TextField,
    status: String,

    // users
    me: Username,
    peer: Username,
    /// oldest messages come first
    history: Vec<Update>,

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
            sending: false,

            input: TextField::new(),
            status: String::new(),

            me,
            peer,
            history: Vec::new(),

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
                match tx.send(AppEvent::Input(event)) {
                    Ok(()) => {}
                    Err(_) => break,
                }
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
                        self.me.clear();
                    }
                }
            }
            Err(e) => {
                self.error(format!("{:?}", e));
            }
        }
    }

    fn spawn_updates_loop(&self) {
        let tx = self.events.0.clone();
        let me = self.me.clone();
        thread::spawn(move || {
            loop {
                let method = GetUpdates { username: me.clone() };
                let answer = method.invoke(&Connection::default());
                let event = match answer {
                    Ok(updates) => AppEvent::Updates(updates),
                    Err(e) => AppEvent::Log { error: true, message: format!("GetUpdates error: {:?}", e) },
                };
                match tx.send(event) {
                    Ok(()) => {}
                    Err(_) => break,
                }
                thread::sleep(::std::time::Duration::from_millis(500));
            }
        });
    }

    pub fn event_loop(&mut self) -> Result<()> {
        self.set_up();
        self.login();
        self.spawn_updates_loop();

        while self.state != State::Exit {
            self.resize_maybe()?;

            {
                let t = &mut *self.t.borrow_mut();
                self.draw(t)?;
                // drop t
            }

            let event = self.events.1.recv()?;
            self.handle_app_event(event)?;
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

    fn draw(&self, t: &mut Terminal<TermionBackend>) -> Result<()> {
        Group::default()
            .direction(Direction::Vertical)
            .margin(0)
            .sizes(&[Size::Min(0), Size::Fixed(2), Size::Fixed(2)]) // status bar at the bottom
            .render(t, &self.size, |t, chunks| {
                if self.history.is_empty() {
                    // logo
                    let logo = ::logo::logo_for_size(chunks[0]);
                    Paragraph::default()
                        .text(&logo)
                        .style(Style::default()
                            .fg(Color::LightMagenta)
                            .modifier(Modifier::Bold))
                        .raw(true)
                        .wrap(false)
                        .block(Block::default()
                            .borders(border::ALL))
                        .render(t, &chunks[0]);
                } else {
                    Paragraph::default()
                        .text(&format_updates(&self.history))
                        .raw(false)
                        .wrap(true)
                        .block(Block::default()
                            .borders(border::ALL))
                        .render(t, &chunks[0]);
                }
                LineEdit::default()
                    .label("Text")
                    .text(&self.input.buffer)
                    .cursor(self.input.cursor)
                    .focus(!self.sending)
                    .render(t, &chunks[1]);

                StatusBar::default()
                    .message(&self.status)
                    .error(self.state == State::Error)
                    .render(t, &chunks[2]);
            });
        t.draw()?;
        Ok(())
    }

    fn handle_app_event(&mut self, event: AppEvent) -> Result<()> {
        match event {
            AppEvent::Input(ev) => self.handle_input(ev)?,
            AppEvent::Log { message, error } => {
                if error {
                    self.state = State::Error;
                } else {
                    self.state = State::LoggedIn;
                }
                self.status = message;
            }
            AppEvent::SentText(msg) => {
                self.history.push(Update::TextUpdate {
                    from: msg.from,
                    to: msg.to,
                    payload: msg.payload,
                });

                self.input.reset();
                self.sending = false;
            }
            AppEvent::SendFailed { reason } => {
                self.sending = false;
                self.state = State::Error;
                self.status = reason;
            }
            AppEvent::Updates(updates) => {
                let Updates::Updates { updates } = updates;
                self.history.extend(updates);
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_input(&mut self, event: Event) -> Result<()> {
        match event {
            Event::Key(Key::Esc) => {
                if !self.status.is_empty() {
                    self.status.clear();
                } else {
                    self.state = State::Exit;
                }
            }
            Event::Key(Key::Char('\n')) => {
                self.send()?;
            }
            Event::Key(Key::Ctrl('l')) => { /* redraw */ }
            event => {
                if !self.sending {
                    self.input.handle_event(event);
                }
            }
        }
        Ok(())
    }

    fn send(&mut self) -> Result<()> {
        self.sending = true;

        if self.input.buffer.len() == 0 {
            self.events.0.send(AppEvent::SendFailed { reason: "Message is empty".into() })?;
            return Ok(());
        }

        let content = self.input.buffer.clone();
        let me = self.me.clone();
        let peer = self.peer.clone();

        let tx = self.events.0.clone();

        info(&tx, "Send message: compressing...");

        thread::spawn(move || {
            thread::sleep(::std::time::Duration::from_millis(500));
            let method = SendText {
                from: me,
                to: peer,
                payload: Data::from_bytes(
                    content.as_bytes(),
                    Compression::Rle,
                    Coding::Hamming,
                ).unwrap(),
            };

            info(&tx, "Send message: compressed and encoded");

            let result = method.invoke(&Connection::default());
            thread::sleep(::std::time::Duration::from_millis(500));

            match result {
                Ok(answer) => {
                    if answer == false {
                        tx.send(AppEvent::SendFailed { reason: "Send message: server error".into() }).unwrap();
                    } else {
                        info(&tx, "Send message: done");
                        tx.send(AppEvent::SentText(method)).unwrap();
                    }
                }
                Err(e) => {
                    tx.send(AppEvent::SendFailed { reason: format!("Send message: Error: {:?}", e) }).unwrap();
                }
            }
        });
        Ok(())
    }
}

fn format_updates(updates: &[Update]) -> String {
    let mut s = String::new();

    for update in updates {
        match *update {
            Update::TextUpdate { ref from, ref to, ref payload } => {
                s.push_str(&format!("[{} to {}]: ", colorize_username(from), colorize_username(to)));

                let msg = match payload.clone().into_bytes() {
                    Ok(vec) => match String::from_utf8(vec) {
                        Ok(s) => escape_brackets(s),
                        Err(_) => "{red UTF-8 error}".into(),
                    },
                    Err(_) => "{red decoding error}".into(),
                };
                s.push_str(&msg);
                s.push_str("\n");
            }
            _ => {}
        }
    }

    s
}

fn escape_brackets(s: String) -> String {
    s.replace("\\", "\\\\").replace("{", "\\{")
}

fn color_for_name(name: &str) -> &'static str {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut h = DefaultHasher::new();
    name.hash(&mut h);
    let hash = h.finish();

    const COLORS: &[&str] = &[
        "yellow",
        "magenta",
        "cyan",
        "light_red",
        "light_magenta",
    ];
    COLORS[(1 + hash) as usize % COLORS.len()]
}

fn colorize_username(s: &Username) -> String {
    let color = color_for_name(s);
    format!("{{fg={} {}}}", color, s)
}

fn info<I: Into<String>>(tx: &Sender<AppEvent>, message: I) {
    let message = message.into();
    tx.send(AppEvent::Log { message, error: false }).unwrap();
}

#[allow(unused)]
fn error<I: Into<String>>(tx: &Sender<AppEvent>, message: I) {
    let message = message.into();
    tx.send(AppEvent::Log { message, error: true }).unwrap();
}

impl Drop for App {
    fn drop(&mut self) {
        let t = &mut *self.t.borrow_mut();

        t.show_cursor().unwrap();
        t.clear().unwrap();
    }
}