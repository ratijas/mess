use super::*;

impl ServerMethod<App> for UploadFile {
    fn handle(self, app: &mut App) -> bool {
        if self.to == self.from || !app.users.contains_key(&self.to) { return false; }

        if !app.pending.remove(&self.file_id) ||
            app.files.contains_key(&self.file_id) { return false; };

        app.files.insert(self.file_id.clone(), File { meta: self.meta.clone(), payload: self.payload });

        app.get_or_new_user(self.to.clone())
           .inbox
           .push_back(Update::FileUpdate {
               from: self.from,
               to: self.to,
               meta: self.meta,
               file_id: self.file_id,
           });
        true
    }
}
