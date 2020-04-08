use super::*;

impl ServerMethod<App> for SendText {
    fn handle(self, app: &mut App) -> bool {
        if self.to == self.from || !app.users.contains_key(&self.to) {
            return false;
        }
        app.get_or_new_user(self.to.clone())
           .inbox
           .push_back(Update::TextUpdate {
               from: self.from,
               to: self.to,
               payload: self.payload,
           });
        true
    }
}
