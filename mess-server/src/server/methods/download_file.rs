use super::*;

impl ServerMethod<App> for DownloadFile {
    fn handle(self, app: &mut App) -> DownloadedFile {
        match app.files.remove(&self.file_id) {
            Some(file) => DownloadedFile::File { data: file.payload },
            None => DownloadedFile::EmptyFile {},
        }
    }
}
