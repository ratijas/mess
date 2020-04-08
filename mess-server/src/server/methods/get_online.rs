use super::*;

impl ServerMethod<App> for GetOnline {
    fn handle(self, app: &mut App) -> Self::Answer {
        Online::Online { users: app.users.keys().cloned().collect() }
    }
}
