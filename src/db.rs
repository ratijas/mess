use std::io::{self, Read};
use std::fs;
use std::result;

use rusqlite::*;

use mime::Mime;
use mime_guess;

const PATH: &str = "./stats.db";
const SCHEMA: &str = "./data/schema.sql";


#[derive(Debug, Clone)]
pub struct File {
    pub file_name: String,
    pub file_type: Mime,
    pub file_size: i32,
}

#[derive(Debug)]
pub struct Compression {
    pub file_name: String,
    pub compression: String,
    pub compress_rate: f64,
    pub size_compressed: i32,
}

#[derive(Debug)]
pub struct Coding {
    pub file_name: String,
    pub compression: String,
    pub coding_name: String,
    pub noise_rate: String,
    pub redundancy_rate: f64,
    pub size_decoded: i32,
    pub size_encoded: i32,
    pub corrected: i32,
    pub detected: i32,
    pub not_corrected: i32,
}

fn connection() -> Result<Connection> {
    Connection::open(PATH)
}

pub fn create_schema() -> result::Result<(), ()> {
    let conn = connection().map_err(drop)?;

    let mut f = fs::File::open(SCHEMA).map_err(drop)?;
    let mut schema = String::new();
    f.read_to_string(&mut schema).map_err(drop)?;

    conn.execute_batch(&schema).map_err(drop)?;

    Ok(())
}


impl File {
    pub fn for_dir_entry(entry: &fs::DirEntry) -> io::Result<File> {
        Ok(File {
            file_name: entry.path().into_os_string().into_string().map_err(|_| io::Error::from(io::ErrorKind::Other))?,
            file_type: mime_guess::guess_mime_type(entry.path()),
            file_size: entry.metadata()?.len() as i32,
        })
    }

    #[allow(unused)]
    pub fn load<I: AsRef<str>>(file_name: I) -> Result<File> {
        let conn = connection()?;
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
    }

    pub fn save(&self) -> Result<()> {
        let conn = connection()?;
        let sql = "
        INSERT OR REPLACE INTO `file` (`file_name`, `file_type`, `file_size`)
                        VALUES (?1, ?2, ?3)
        ";
        let mut stmt = conn.prepare_cached(sql)?;
        stmt.execute(&[
            &self.file_name,
            &format!("{}", self.file_type),
            &self.file_size]
        )?;
        Ok(())
    }
}

impl Compression {
    pub fn save(&self) -> Result<()> {
        let conn = connection()?;
        let sql = "
        INSERT OR REPLACE INTO compression (file_name,
                                            compression,
                                            compress_rate,
                                            size_compressed)
                        VALUES (?1, ?2, ?3, ?4)
        ";
        let mut stmt = conn.prepare_cached(sql)?;
        stmt.execute(&[
            &self.file_name,
            &self.compression,
            &self.compress_rate,
            &self.size_compressed,
        ])?;
        Ok(())
    }
}

impl Coding {
    #[allow(unused)]
    pub fn load(file_name: &str,
                compression: &str,
                coding_name: &str,
                noise_rate: &str) -> Result<Coding>
    {
        let conn = connection()?;
        let sql = "\
            SELECT redundancy_rate, size_decoded, size_encoded, corrected, detected, not_corrected
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
                }
            })
    }

    pub fn save(&self) -> Result<()> {
        let conn = connection()?;
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
                                       not_corrected)
                        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
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
        ])?;
        Ok(())
    }
}
