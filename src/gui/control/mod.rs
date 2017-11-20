mod text_field;

pub use self::text_field::TextField;

use imports::*;

pub trait Responder {
    /// return true if this controller has captured the event during bubble down or bubble up phase.
    fn handle_event(&mut self, event: Event) -> bool;
}
