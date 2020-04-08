//! Concepts of view controller, view, model and delegate.
//!
//! View controller is stateful object capable of handling events (e.g. input),
//! it also can implement one or more delegate protocols. Composition of controllers
//! makes up a tree, which is an acyclic (non-recursive) directed graph. It makes it possible
//! to attach parent's lifetime to descendant controllers.
//!
//! View is a disposable object, created from the corresponding controller,
//! configured, rendered and thrown away. In terms of `tui` crate, View is a [Widget].
//!
//! Normally (?), each view type corresponds to exactly one view controller.
//!
//! Model is storage for some data, and methods to manipulate it. `String` is a good example.
//! `Vec<User>` will also do.
//!
//! Delegate protocol is (in terms of Rust programming language) a trait, where each method
//! corresponds to some controller's event or request.
//!
//! Controller may possess interface to work with as many delegate protocols as desired.
//!
//! # What about lifetimes?
//!
//! ## View
//!
//! View takes an immutable reference to its controller, so it's always `'a`.
//!
//! ## Delegate
//!
//! A reference or references to delegates (if any) must be hold inside controller. Such structure
//! prevents us from using delegates as mutable objects from now on, which is unacceptable.
//! Instead delegates could be implemented as separate types, while controller holding `RefCel`
//! with a delegate and passing around `&'a` references to it.
//!
//! Another approach would be to use `Arc<RefCell<T>>` wherever possible.
//!
//! # Event handling
//!
//! View controller's primary job is to handle input. It exposes via `ViewController` trait several
//! methods for that. When an event is emitted by top-level event system, there is some view
//! controller which must handle it. Search starts with root view controller and goes down the
//! chain of "active" subview until some controller handles it or there's no more active subview
//! left.
//!
//! Schematically:
//!
//! ```python
//! def handle_event(self, event):
//!     if self.bubble_down(event):
//!         # event handled during "bubble-down" phase
//!         return True
//!     else if self.active_subview is not None:
//!         # going down
//!         if self.active_subview.handle_event(event):
//!             # event handled by subview
//!             return True
//!     # both else cases
//!     return self.bubble_up(event)
//! ```
//!
//! [Widget]: https://docs.rs/tui/0.1.3/tui/widgets/trait.Widget.html

mod imports {
    pub use std::cell::RefCell;
    pub use std::sync::{Arc, Weak};

    pub use tui::backend::{Backend, TermionBackend};
    pub use tui::buffer::{Buffer, Cell};
    pub use tui::layout::{Direction, Group, Rect, Size};
    pub use tui::style::{Color, Modifier, Style};
    pub use tui::widgets::{border, Block, List, SelectableList, Paragraph, Widget};
    pub use tui::Terminal;

    pub use termion::event::{Event, Key};
}

use self::imports::*;


mod boxes {
    use super::*;

    pub type Boxed<T> = Arc<RefCell<Box<T>>>;
    pub type WeakBoxed<T> = Weak<RefCell<Box<T>>>;

    pub trait BoxedExt {
        fn boxed(self) -> Boxed<Self>;
    }

    impl<T> BoxedExt for T {
        fn boxed(self) -> Boxed<Self> {
            boxed(self)
        }
    }

    pub fn boxed<T>(t: T) -> Arc<RefCell<Box<T>>> {
        Arc::new(RefCell::new(Box::new(t)))
    }
}

pub use self::boxes::*;

#[macro_export]
macro_rules! boxed_as {
    ($t:ty, $value:expr) => {
        Arc::new(RefCell::new(Arc::try_unwrap($value).map_err(drop).expect("rebox").into_inner() as Box<$t>))
    };
}



#[derive(Default)]
pub struct ViewControllerImpl {
    /// Weak reference to the parent view controller
    parent_view_controller: Weak<RefCell<Box<ViewController>>>,

    // /// Child view controllers that
    // subviews: Vec<Arc<RefCell<Box<ViewController>>>>,

    /// Weak reference to the next child responder down the hierarchy
    active_child_view_controller: Weak<RefCell<Box<ViewController>>>,
}


pub trait ViewController: Widget {
    fn as_view_controller(&self) -> &ViewControllerImpl;
    fn as_view_controller_mut(&mut self) -> &mut ViewControllerImpl;

    /// return true if this controller has captured the event during bubble down or bubble up phase.
    fn handle_event(&mut self, event: Event, bubble_down: bool) -> bool;

    /// Weak reference to the parent_view_controller view controller
    fn parent_view_controller(&self) -> WeakBoxed<ViewController>;

    fn set_parent_view_controller(&mut self, parent: WeakBoxed<ViewController>);

    /// Weak reference to the next child responder down the hierarchy
    fn active_child_view_controller(&self) -> WeakBoxed<ViewController>;

    fn set_active_child_view_controller(&mut self, child: Boxed<ViewController>);

    fn render_on_termion(&self, t: &mut Terminal<TermionBackend>, area: &Rect);
}

default impl<T: Widget + Sized> ViewController for T {
    default fn as_view_controller(&self) -> &ViewControllerImpl { unimplemented!() }
    default fn as_view_controller_mut(&mut self) -> &mut ViewControllerImpl { unimplemented!() }
    default fn handle_event(&mut self, event: Event, bubble_down: bool) -> bool { unimplemented!() }

    default fn parent_view_controller(&self) -> WeakBoxed<ViewController> {
        self.as_view_controller().parent_view_controller.clone()
    }
    default fn set_parent_view_controller(&mut self, parent: WeakBoxed<ViewController>) {
        self.as_view_controller_mut().parent_view_controller = parent;
    }
    default fn active_child_view_controller(&self) -> WeakBoxed<ViewController> {
        self.as_view_controller().active_child_view_controller.clone()
    }
    default fn set_active_child_view_controller(&mut self, child: Boxed<ViewController>) {
        self.as_view_controller_mut().active_child_view_controller = Arc::downgrade(&child);
    }

    /* final */ fn render_on_termion(&self, t: &mut Terminal<TermionBackend>, area: &Rect) {
        self.render(t, area);
    }
}

macro_rules! view_controller_impl {
    () => {
        fn as_view_controller(&self) -> &ViewControllerImpl { &self._inner }
        fn as_view_controller_mut(&mut self) -> &mut ViewControllerImpl { &mut self._inner }
        fn render_on_termion(&self, t: &mut Terminal<TermionBackend>, area: &Rect) {
            self.render(t, area);
        }
    };
}

pub mod button {
    use super::*;

    #[derive(Default)]
    pub struct Button {
        _inner: ViewControllerImpl,
        pub title: String,
        pub highlighted: bool,
    }

    // builder pattern
    impl Button {
        pub fn title(mut self, title: String) -> Button {
            self.title = title;
            self
        }

        pub fn highlighted(mut self, highlighted: bool) -> Button {
            self.highlighted = highlighted;
            self
        }
    }

    impl ViewController for Button {
        view_controller_impl!();

        fn handle_event(&mut self, event: Event, bubble_down: bool) -> bool {
            match event {
                Event::Key(Key::Char('\t')) => {
                    self.highlighted = !self.highlighted;
                    true
                }
                _ => false,
            }
        }
    }

    impl Widget for Button {
        fn draw(&self, area: &Rect, buf: &mut Buffer) {
            Paragraph::default()
                .text(&format!("< {} >", &self.title))
                .block(Block::default().borders(border::ALL))
                .style(Style::default().modifier(if self.highlighted { Modifier::NoInvert } else { Modifier::Invert }))
                .draw(area, buf);
        }
    }
}

pub mod window {
    use super::*;

    pub struct Window {
        _inner: ViewControllerImpl,
        root: Arc<RefCell<Box<ViewController>>>,
        modal: Option<Arc<RefCell<Box<ViewController>>>>,
    }

    impl Window {
        pub fn new() -> Boxed<Window> {
            let root: Boxed<ViewController> = boxed_as!(ViewController, main_view_controller::MainViewController::new());
            let mut window = Window {
                _inner: Default::default(),
                root: Arc::clone(&root),
                modal: None,
            };
            window.set_active_child_view_controller(root);
            window.boxed()
        }
    }

    impl ViewController for Window {
        view_controller_impl!();

        fn handle_event(&mut self, event: Event, bubble_down: bool) -> bool {
            if let Some(ref modal) = self.modal {
                return modal.borrow_mut().handle_event(event, bubble_down);
            }
            false
        }
    }

    impl Widget for Window {
        fn draw(&self, area: &Rect, buf: &mut Buffer) {
            (*self.root).borrow().draw(area, buf);

            if let Some(ref modal) = self.modal {
                (**modal).borrow().draw(area, buf);
            }
        }
    }
}

pub mod main_view_controller {
    use super::*;
    use super::list_view_controller::*;

    pub struct MainViewController {
        _inner: ViewControllerImpl,

        users_online_controller: Boxed<ListViewController>,

        status_bar: Boxed<ViewController>,
    }

    impl MainViewController {
        pub fn new() -> Boxed<MainViewController> {
            let mut status_bar: Boxed<ViewController> = boxed_as!(ViewController, text_field::TextField::new());

            let this = MainViewController {
                _inner: Default::default(),
                users_online_controller: ListViewController::new(
                    boxed_as!(ListViewDataSource,
                            DummyListViewDataSource::new(
                                &["one", "two", "three"],
                                Some(1),
                            )
                        )
                ),
                status_bar: Arc::clone(&status_bar),
            }.boxed();
            //            status_bar.borrow_mut().set_parent_view_controller(Arc::downgrade(&this));
            this.borrow_mut().set_active_child_view_controller(status_bar);
            this
        }
    }

    impl ViewController for MainViewController {
        view_controller_impl!();

        fn handle_event(&mut self, event: Event, bubble_down: bool) -> bool {
            false
        }
    }

    impl Widget for MainViewController {
        fn draw(&self, area: &Rect, buf: &mut Buffer) {
            //            Group::default()
            //                .direction(Direction::Vertical)
            //                .sizes(&[Size::Min(0), Size::Fixed(3)])
            //                .render( ??? );

            self.users_online_controller.borrow().draw(area, buf);
        }
    }
}

pub mod list_view_controller {
    use super::*;

    pub struct ListViewController {
        _inner: ViewControllerImpl,
        data_source: Arc<RefCell<Box<ListViewDataSource>>>,
    }

    /// ListViewDataSource protocol
    pub trait ListViewDataSource {
        fn list_view_number_of_rows(
            &self,
            list_view: &ListViewController,
        ) -> usize;

        fn list_view_cell_for_row_at_index(
            &self,
            list_view: &ListViewController,
            index: usize,
        ) -> Option<String>;

        fn list_view_selection_index(
            &self,
            list_view: &ListViewController,
        ) -> Option<usize>;

        // this actually should be in delegate protocol
        fn list_view_selection_changed(
            &mut self,
            list_view: &ListViewController,
            index: Option<usize>,
        );
    }

    pub struct DummyListViewDataSource {
        pub items: Vec<String>,
        pub selection: Option<usize>,
    }

    pub trait OptionalSelection<'a> {
        fn select_optional(&'a mut self, index: Option<usize>) -> &'a mut Self;
    }


    impl ListViewController {
        pub fn new(data_source: Boxed<ListViewDataSource>) -> Boxed<ListViewController> {
            ListViewController {
                _inner: Default::default(),
                data_source,
            }.boxed()
        }

        fn select_up(&mut self) {
            if 0 == (*self.data_source).borrow().list_view_number_of_rows(&self) { return; }
            let index = (*self.data_source).borrow().list_view_selection_index(&self).unwrap_or(0);
            if index == 0 { return; }

            let new_index = index - 1;
            (*self.data_source).borrow_mut().list_view_selection_changed(&self, Some(new_index));
        }

        fn select_down(&mut self) {
            let n = (*self.data_source).borrow().list_view_number_of_rows(&self);
            if n == 0 { return; }

            let index = (*self.data_source).borrow().list_view_selection_index(&self).unwrap_or(0);
            if n == index { return; }

            let new_index = index + 1;
            (*self.data_source).borrow_mut().list_view_selection_changed(&self, Some(new_index));
        }

        fn items(&self) -> Vec<String> {
            let ds = (*self.data_source).borrow();
            let n = ds.list_view_number_of_rows(&self);
            let mut items = Vec::with_capacity(n);

            for i in 0..n {
                match ds.list_view_cell_for_row_at_index(&self, i) {
                    Some(item) => items.push(item),
                    None => break,
                }
            }
            items
        }
    }

    impl ViewController for ListViewController {
        view_controller_impl!();

        fn handle_event(&mut self, event: Event, bubble_down: bool) -> bool {
            match event {
                Event::Key(Key::Up) => self.select_up(),
                Event::Key(Key::Down) => self.select_down(),
                _ => return false,
            }
            true
        }
    }

    impl Widget for ListViewController {
        fn draw(&self, area: &Rect, buf: &mut Buffer) {
            SelectableList::default()
                .items(&self.items())
                .select_optional((*self.data_source).borrow().list_view_selection_index(&self))
                .highlight_symbol("> ")
                .draw(area, buf);
        }
    }

    impl<'a> OptionalSelection<'a> for SelectableList<'a> {
        fn select_optional(&'a mut self, index: Option<usize>) -> &'a mut SelectableList<'a> {
            match index {
                Some(index) => self.select(index),
                None => self,
            }
        }
    }

    impl DummyListViewDataSource {
        pub fn new<I: AsRef<str>>(items: &[I], selection: Option<usize>) -> Boxed<DummyListViewDataSource> {
            DummyListViewDataSource {
                items: items.iter().map(AsRef::as_ref).map(Into::into).collect(),
                selection,
            }.boxed()
        }
    }

    impl ListViewDataSource for DummyListViewDataSource {
        fn list_view_number_of_rows(&self, list_view: &ListViewController) -> usize {
            self.items.len()
        }

        fn list_view_cell_for_row_at_index(&self, list_view: &ListViewController, index: usize) -> Option<String> {
            self.items.get(index).cloned()
        }

        fn list_view_selection_index(&self, list_view: &ListViewController) -> Option<usize> {
            self.selection
        }

        fn list_view_selection_changed(&mut self, list_view: &ListViewController, index: Option<usize>) {
            self.selection = index;
        }
    }
}

/*
pub mod modal {
    use super::*;

    use super::text_field::TextField;

    /// Stateful view controller
    #[derive(Default)]
    pub struct Modal {
        title: String,
        editor: TextField,
        shadow: bool
    }

    impl Modal {
        pub fn title(mut self, title: String) -> Self {
            self.title = title;
            self
        }

        //    pub fn input(mut self, input: String) -> Self {
        //        self.input = input;
        //        self
        //    }

        pub fn shadow(mut self, shadow: bool) -> Self {
            self.shadow = shadow;
            self
        }

        /// edit line until `key` is the Enter key.  Then return input content.
        pub fn handle(&mut self, key: Key) -> Option<String> {
            match key {
                Key::Char('\n') => Some(self.editor.buffer.clone()),
                _ => {
                    self.editor.handle(key);
                    None
                }
            }
        }

        pub fn render<B: Backend>(&self, t: &mut Terminal<B>, area: &Rect) {
            Group::default()
                .direction(Direction::Horizontal)
                .margin(0)
                .sizes(&[Size::Percent(15), Size::Min(1), Size::Percent(15)])
                .render(t, area, |t, chunks| {
                    Group::default()
                        .direction(Direction::Vertical)
                        .margin(0)
                        .sizes(&[Size::Percent(45), Size::Fixed(5), Size::Percent(45)])
                        .render(t, &chunks[1], |t, chunks| {
                            Block::default()
                                .borders(border::ALL)
                                .border_style(Style::default().bg(Color::DarkGray).fg(Color::DarkGray))
                                .render(t, &chunks[1]);

                            let area = &chunks[1].inner(1);

                            TextField::default()
                                .title(&self.title)
                                .text(self.editor.buffer())
                                .cursor(self.editor.cursor)
                                .render(t, &area);
                        });
                });
        }
    }
}
*/

pub mod text_field {
    use super::*;

    /// Single line of editable text
    #[derive(Default)]
    pub struct TextField {
        _inner: ViewControllerImpl,
        title: String,
        text: String,
        cursor: usize,
    }

    impl TextField {
        pub fn new() -> Boxed<TextField> {
            boxed(Default::default())
        }
        pub fn title<I: AsRef<str>>(mut self, title: I) -> TextField {
            self.title = title.as_ref().into();
            self
        }
        pub fn text<I: AsRef<str>>(mut self, text: I) -> TextField {
            self.text = text.as_ref().into();
            self
        }
        pub fn cursor(mut self, cursor: usize) -> TextField {
            self.cursor = cursor;
            self
        }
    }

    impl ViewController for TextField {
        view_controller_impl!();

        fn handle_event(&mut self, event: Event, _bubble_down: bool) -> bool {
            if let Event::Key(key) = event {
                match key {
                    Key::Char(ch) => {
                        self.text.insert(self.cursor, ch);
                        self.cursor += 1;
                    }

                    Key::Backspace => {
                        if self.cursor > 0 {
                            self.text.remove(self.cursor - 1);
                            self.cursor -= 1;
                        }
                    }
                    Key::Left => {
                        if self.cursor > 0 {
                            self.cursor -= 1;
                        }
                    }
                    Key::Right => {
                        if self.cursor + 1 <= self.text.len() {
                            self.cursor += 1;
                        }
                    }
                    _ => { return false; }
                }
                true
            } else {
                false
            }
        }
    }

    impl Widget for TextField {
        fn draw(&self, area: &Rect, buf: &mut Buffer) {
            // line itself + borders
            if area.height < 3 {
                return;
            }
            let bg = Style::default().bg(Color::Green).fg(Color::Black);

            buf.clear_area(area);

            Paragraph::default()
                .block(
                    Block::default()
                        .borders(border::ALL)
                        .border_style(bg.clone())
                        .title(&self.title)
                        .title_style(bg.clone().modifier(Modifier::Invert))
                )
                .wrap(false)
                .raw(true)
                .text(&self.text)
                .draw(area, buf);

            buf.get_mut((1 + area.left() + self.cursor as u16).min(area.right() - 1),
                        1 + area.top())
               .style.modifier = Modifier::Invert;
        }
    }

    pub trait BufferCleaner {
        fn clear_area(&mut self, area: &Rect);
    }

    impl BufferCleaner for Buffer {
        fn clear_area(&mut self, area: &Rect) {
            for y in area.top()..area.bottom() {
                for x in area.left()..area.right() {
                    self.get_mut(x, y).symbol = " ".into();
                }
            }
        }
    }
}
