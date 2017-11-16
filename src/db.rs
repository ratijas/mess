use std::io::{self, Read};
use std::fs;
use std::result;
use std::sync::Mutex;

use rusqlite::*;

use mime::Mime;
use mime_guess;

const PATH: &str = "./stats.db";
const SCHEMA: &str = "./data/schema.sql";

lazy_static! {
    static ref GLOBAL_CONNECTION: Mutex<Connection> = Mutex::new(Connection::open(PATH).unwrap());
}


#[derive(Debug, Clone)]
pub struct File {
    pub file_name: String,
    pub file_type: Mime,
    pub file_size: i64,
}

#[derive(Debug)]
pub struct Compression {
    pub file_name: String,
    pub compression: String,
    pub compress_rate: f64,
    pub size_compressed: i64,
    pub time_compress: i64,
    pub time_decompress: Option<i64>,
}

#[derive(Debug)]
pub struct Coding {
    pub file_name: String,
    pub compression: String,
    pub coding_name: String,
    pub noise_rate: String,
    pub redundancy_rate: f64,
    pub size_decoded: i64,
    pub size_encoded: i64,
    pub corrected: i64,
    pub detected: i64,
    pub not_corrected: i64,
    pub time_encode: i64,
    pub time_decode: i64,
}

pub fn connection<F, T, E>(f: F) -> result::Result<T, E>
    where F: FnOnce(&Connection) -> result::Result<T, E>
{
    let guard = GLOBAL_CONNECTION.lock().unwrap();
    f(&*guard)
}

pub fn create_schema() -> ::Result<()> {
    connection(|conn| {
        let mut f = fs::File::open(SCHEMA)?;
        let mut schema = String::new();
        f.read_to_string(&mut schema)?;

        conn.execute_batch(&schema)?;

        Ok(())
    })
}


impl File {
    pub fn for_dir_entry(entry: &fs::DirEntry) -> io::Result<File> {
        Ok(File {
            file_name: entry.path().into_os_string().into_string().map_err(|_| io::Error::from(io::ErrorKind::Other))?,
            file_type: mime_guess::guess_mime_type(entry.path()),
            file_size: 8 * entry.metadata()?.len() as i64,
        })
    }

    #[allow(unused)]
    pub fn load<I: AsRef<str>>(file_name: I) -> Result<File> {
        connection(|conn| {
            let sql = "\
                SELECT file_type, file_size
                  FROM file
                 WHERE file_name = ?1
            ";
            let mut stmt = conn.prepare_cached(sql)?;
            stmt.query_row(&[&file_name.as_ref()], |row| {
                File {
                    file_name: file_name.as_ref().into(),
                    file_type: mime_guess::guess_mime_type(row.get::<_, String>(0)),
                    file_size: row.get(1),
                }
            })
        })
    }

    pub fn save(&self) -> Result<()> {
        connection(|conn| {
            let sql = "
                INSERT OR REPLACE INTO file (file_name, file_type, file_size)
                                VALUES (?1, ?2, ?3)
            ";
            let mut stmt = conn.prepare_cached(sql)?;
            stmt.execute(&[
                &self.file_name,
                &format!("{}", self.file_type),
                &self.file_size]
            )?;
            Ok(())
        })
    }
}

impl Compression {
    pub fn save(&self) -> Result<()> {
        connection(|conn| {
            let sql = "
            INSERT OR REPLACE INTO compression (file_name,
                                                compression,
                                                compress_rate,
                                                size_compressed,
                                                time_compress,
                                                time_decompress)
                            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            ";
            let mut stmt = conn.prepare_cached(sql)?;
            stmt.execute(&[
                &self.file_name,
                &self.compression,
                &self.compress_rate,
                &self.size_compressed,
                &self.time_compress,
                &self.time_decompress,
            ])?;
            Ok(())
        })
    }
}

impl Coding {
    #[allow(unused)]
    pub fn load(file_name: &str,
                compression: &str,
                coding_name: &str,
                noise_rate: &str) -> Result<Coding>
    {
        connection(|conn| {
            let sql = "\
                SELECT redundancy_rate, size_decoded, size_encoded, corrected, detected, not_corrected, time_encode, time_decode
                  FROM coding
                 WHERE file_name = ?1
                   AND coding_name = ?2
                   AND noise_rate = ?3
            ";
            let mut stmt = conn.prepare_cached(sql)?;
            stmt.query_row(
                &[&file_name, &coding_name, &noise_rate],
                |row| {
                    Coding {
                        file_name: file_name.into(),
                        compression: compression.into(),
                        coding_name: coding_name.into(),
                        noise_rate: noise_rate.into(),
                        redundancy_rate: row.get(0),
                        size_decoded: row.get(1),
                        size_encoded: row.get(2),
                        corrected: row.get(3),
                        detected: row.get(4),
                        not_corrected: row.get(5),
                        time_encode: row.get(6),
                        time_decode: row.get(7),
                    }
                })
        })
    }

    pub fn save(&self) -> Result<()> {
        connection(|conn| {
            let sql = "
            INSERT OR REPLACE INTO coding (file_name,
                                           compression,
                                           coding_name,
                                           noise_rate,
                                           redundancy_rate,
                                           size_decoded,
                                           size_encoded,
                                           corrected,
                                           detected,
                                           not_corrected,
                                           time_encode,
                                           time_decode)
                            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
            ";
            let mut stmt = conn.prepare_cached(sql)?;
            stmt.execute(&[
                &self.file_name,
                &self.compression,
                &self.coding_name,
                &self.noise_rate,
                &self.redundancy_rate,
                &self.size_decoded,
                &self.size_encoded,
                &self.corrected,
                &self.detected,
                &self.not_corrected,
                &self.time_encode,
                &self.time_decode,
            ])?;
            Ok(())
        })
    }
}
