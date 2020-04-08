BEGIN TRANSACTION;

/* all sizes are in bits */

DROP TABLE IF EXISTS `file`;
CREATE TABLE `file` (
  `file_name` TEXT    NOT NULL PRIMARY KEY,
  `file_type` TEXT    NOT NULL,
  `file_size` INTEGER NOT NULL
);

DROP TABLE IF EXISTS `compression`;
CREATE TABLE `compression` (
  `file_name`       TEXT    NOT NULL,
  `compression`     TEXT    NOT NULL CHECK (`compression` IN ('huffman', 'rle', 'shannon')),
  `compress_rate`   REAL    NOT NULL,
  `size_compressed` INTEGER NOT NULL,
  `time_compress`   INTEGER NOT NULL,
  `time_decompress` INTEGER, -- could be NULL if decompression failed
  PRIMARY KEY (`file_name`, `compression`),
  FOREIGN KEY (`file_name`) REFERENCES `file` (`file_name`)
);

DROP TABLE IF EXISTS `coding`;
CREATE TABLE `coding` (
  `file_name`       TEXT    NOT NULL,
  `compression`     TEXT    NOT NULL,
  `coding_name`     TEXT    NOT NULL CHECK (`coding_name` IN ('hamming', 'parity', 'r3', 'r5')),
  `noise_rate`      TEXT    NOT NULL CHECK (`noise_rate` IN ('0.01', '0.05', '0.15')),
  `redundancy_rate` REAL    NOT NULL,
  `size_decoded`    INTEGER NOT NULL, /* bazicali, duplicate of `compression`.`size_compressed` */
  `size_encoded`    INTEGER NOT NULL,
  `corrected`       INTEGER NOT NULL,
  `detected`        INTEGER NOT NULL CHECK (`corrected` <= `detected`),
  `not_corrected`   INTEGER NOT NULL CHECK (`not_corrected` <= `size_decoded`),
  `time_encode`     INTEGER NOT NULL,
  `time_decode`     INTEGER NOT NULL,
  PRIMARY KEY (`file_name`, `compression`, `coding_name`, `noise_rate`),
  FOREIGN KEY (`file_name`) REFERENCES `file` (`file_name`),
  FOREIGN KEY (`file_name`, `compression`) REFERENCES `compression` (`file_name`, `compression`)
);

COMMIT;
