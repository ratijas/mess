use super::*;

impl ServerMethod<App> for GetUpdates {
    fn handle(self, app: &mut App) -> Self::Answer {
        let user: &mut User = app.get_or_new_user(self.username.clone());
        let updates: Vec<Update> = user.inbox.drain(..).collect();
        Updates::Updates { updates }
    }
}
