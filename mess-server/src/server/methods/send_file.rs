use super::*;

impl ServerMethod<App> for SendFile {
    fn handle(self, app: &mut App) -> FileId {
        let file_id = FileId::FileId(app.last_id);
        app.last_id += 1;
        app.pending.insert(file_id.clone());
        file_id
    }
}
