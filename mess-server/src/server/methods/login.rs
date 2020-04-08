use super::*;

impl ServerMethod<App> for Login {
    fn handle(self, app: &mut App) -> Self::Answer {
        let username = self.username;

        if !User::validate_username(&username) {
            return LoginResult::LoginErr;
        }

        let _ = app.get_or_new_user(username.clone());

        LoginResult::LoginOk { username }
    }
}
