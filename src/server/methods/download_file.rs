use super::*;

impl ServerMethod<App> for DownloadFile {
    fn handle(self, app: &mut App) -> Data {
        // TODO: Empty result
        match app.files.remove(&self.file_id) {
            Some(file) => {
                file.payload
            }
            None => {
                error!("file does not exist on server: {:?}", self.file_id);
                panic!();
            }
        }
    }
}
