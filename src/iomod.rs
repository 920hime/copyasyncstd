/**
 * Copyright (C) 2023 awk4j - https://ja.osdn.net/projects/awk4j/
 * <p>
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; either version 3 of the License, or
 * (at your option) any later version.
 * <p>
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 * <p>
 * You should have received a copy of the GNU General Public License
 * along with this program. If not, see <http://www.gnu.org/licenses/>.
 */
use std::fs;
use std::fs::canonicalize;
use std::fs::File;
use std::fs::Metadata;
use std::io::prelude::*;
use std::io::Result;
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};

use std::time::SystemTime;

/**
// Path::is_file()
// Path::is_dir()

// Path <--> PathBuf
// let path: &Path = &path_buf;
// let path_buf: PathBuf = path.to_path_buf();
 */

// https://qiita.com/DeliciousBar/items/de686ade39b00960df61
/**
 * get absolute path (canonicalize)
 * Windows上では「\\?\D:\foo」のような「UNC path」を返す
 * Windowsは「\\?\」で始まるパスは解釈処理をせず、そのまま扱う
 *（ただしAPIが成功するとは限らない）
 */
pub fn absolute_path<P: AsRef<Path>>(path: P) -> String {
    let p: &Path = path.as_ref();
    let abs_path = match canonicalize(p) {
        Err(e) => panic!("{}", error("absolute", p, e.to_string())),
        Ok(x) => path_to_string(&x),
    };
    abs_path
}

/**
 * Path -> String
 */
pub fn path_to_string<P: AsRef<Path>>(path: P) -> String {
    let p: &Path = path.as_ref();
    match p.to_str() {
        None => panic!("Path is not a valid UTF-8 sequence"),
        Some(x) => x.to_string(),
    }
}
pub fn path_to_unix<P: AsRef<Path>>(path: P) -> String {
    path_to_string(path).replace(r"\", r"/")
}

/**
 * mkdir - 深い階層のディレクトリを一気に作成
 */
pub fn mkdir<P: AsRef<Path>>(path: P) -> () {
    let p: &Path = path.as_ref();
    if !p.is_dir() {
        match fs::create_dir_all(p) {
            // 既に存在するファイルを作成することはできません
            // 指定されたパスが見つかりません - Z:/foo
            Err(e) => panic!("{}", error("mkdir", p, e.to_string())),
            Ok(_) => (),
        }
    }
    ()
}

/**
 * remove directory - フォルダをファイルを含めてまるごと削除
 */
pub fn remove_dir_all<P: AsRef<Path>>(path: P) -> () {
    let p: &Path = path.as_ref();
    if p.is_dir() {
        match fs::remove_dir_all(p) {
            Err(e) => panic!("{}", error("rmdir", p, e.to_string())),
            Ok(_) => (),
        }
    }
    ()
}

/**
 * remove file
 */
pub fn _remove_file<P: AsRef<Path>>(path: P) -> () {
    let p: &Path = path.as_ref();
    if p.is_file() {
        match fs::remove_file(p) {
            Err(e) => panic!("{}", error("rmfile", p, e.to_string())),
            Ok(_) => (),
        }
    }
    ()
}

/**
 * get file name
 */
pub fn get_filename<P: AsRef<Path>>(path: P) -> String {
    let p: &Path = path.as_ref();
    let x = p.file_name().unwrap().to_string_lossy();
    x.to_string()
}

// https://runebook.dev/ja/docs/rust/std/fs/fn.copy
/**
 * copy from to -> length
 */
pub fn _copy<P: AsRef<Path>>(from: P, to: P) -> u64 {
    let f: &Path = from.as_ref();
    let t: &Path = to.as_ref();
    match fs::copy(f, t) {
        Err(e) => panic!("{}", error("copy", f, e.to_string())),
        Ok(len) => len, // 指定されたファイルが見つかりません
    }
}

/**
 * rename from to - rename 関数を使用した 爆速 move の実装
 *
 * Unixでは、 fromがディレクトリであれば、 toも(空の)ディレクトリでなければなりません。
 * fromがディレクトリでない場合、 toもまたディレクトリであってはならない。
 * 一方、Windowsでは、 fromは何でもよいが、 toはディレクトリであってはならない。
 */
/// from が示すファイル／ディレクトリを、to のパスへ移動（とリネーム）します。
/// to には移動後のファイル／ディレクトリ名を含んでいる必要があります。
/// to が親ディレクトリを持つ場合、そのディレクトリを先に作成する必要が有ります。
/// to が別のマウントポイント(または別ドライブ)にある場合、機能しません。
/// https://runebook.dev/ja/docs/rust/std/fs/fn.rename
pub fn _rename_file<P: AsRef<Path>>(from: P, to: P) -> () {
    let f: &Path = from.as_ref();
    let t: &Path = to.as_ref();
    match fs::rename(f, t) {
        Err(e) => panic!("{}", error("rename", f, e.to_string())),
        Ok(_) => (), // ファイルを別のディスク ドライブに移動できません
    }
}

// https://runebook.dev/ja/docs/rust/std/fs/struct.metadata
/**
 * get metadata - length    
 */
pub fn get_meta_len<P: AsRef<Path>>(path: P) -> u64 {
    let p: &Path = path.as_ref();
    let metadata: Result<Metadata> = fs::metadata(p);
    metadata.expect("REASON").len()
}
// get metadata - modified
pub fn get_meta_modified<P: AsRef<Path>>(path: P) -> SystemTime {
    let p: &Path = path.as_ref();
    let metadata: Result<Metadata> = fs::metadata(p);
    if let Ok(time) = metadata.expect("REASON").modified() {
        return time;
    } else {
        panic!("Not supported on this platform");
    }
}

/**
 * change directory - dir, foo/var --> dir/var
 */
pub fn _chg_dir(dir: &str, file: &str) -> String {
    let path: &Path = Path::new(file); // ../foo/var
    let name: String = get_filename(path); // var
    let path_buf: PathBuf = PathBuf::from(dir).join(name); // dir/var
    let x: &Path = &path_buf;
    path_to_string(x)
}

/**
 * Open reader
 */
pub fn _open_reader(path: &str) -> (FD, BufReader<File>) {
    let file: File = match File::open(path) {
        Err(e) => panic!("{}", error("Read Error", path, e.to_string())),
        Ok(x) => x,
    };
    let reader: BufReader<File> = BufReader::new(file);
    let fd = FD {
        path: path.to_string().to_owned(),
        buf: String::new(),
        line_number: 0,
        eof: false,
    };
    (fd, reader)
}
// Read line
pub fn _read(mut fd: FD, mut reader: BufReader<File>) -> (FD, BufReader<File>) {
    let mut buf = String::with_capacity(_BUF_CAPACITY);
    let mut len: usize = 0;
    if !fd.eof {
        let rs = reader.read_line(&mut buf);
        len = match rs {
            Err(e) => panic!("{}", error("Read Error", &fd.path, e.to_string())),
            Ok(x) => x,
        };
        if len > 0 {
            fd.line_number += 1;
        }
    }
    fd.buf = buf.trim_end().to_string();
    fd.eof = len == 0; // 通常は少なくとも \n がある
    (fd, reader)
}
const _BUF_CAPACITY: usize = 192;

/**
 * Open writer
 */
pub fn _open_writer(path: &str) -> (FD, BufWriter<File>) {
    let file: File = match File::create(path) {
        Err(e) => panic!("{}", error("Write Error", path, e.to_string())),
        Ok(x) => x,
    };
    let writer: BufWriter<File> = BufWriter::new(file);
    let fd = FD {
        path: path.to_string().to_owned(),
        buf: String::new(),
        line_number: 0,
        eof: false, // unused
    };
    (fd, writer)
}
// Write line
pub fn _write(mut fd: FD, mut writer: BufWriter<File>) -> (FD, BufWriter<File>) {
    let rs: Result<_> = write!(writer, "{}\n", fd._get_buf());
    let _rs = match rs {
        Err(e) => panic!("{}", error("Write Error", &fd.path, e.to_string())),
        Ok(x) => x,
    };
    fd.line_number += 1;
    (fd, writer)
}

// File descriptor (Read/Write) - 構造体、クローン可能
#[derive(Debug, Clone)] // String は Copy を実装できない
pub struct FD {
    pub path: String,     // used in error messages
    pub buf: String,      // Read/Write
    pub line_number: u32, // Number of rows processed
    pub eof: bool,        // Read
}
impl FD {
    pub fn _get_buf(&self) -> String {
        self.buf.clone()
    }
}

// Test Reader
pub fn _reader_test01(path: &str) {
    let (mut rfd, mut reader) = _open_reader(path);
    while !rfd.eof {
        let (_rfd, _reader) = _read(rfd, reader);
        reader = _reader;
        rfd = _rfd;
        if !rfd.eof {
            println!(". {}", rfd._get_buf());
        }
    }
}
// Test Reader/Writer
pub fn _writer_test01(input: &str, output: &str) -> u32 {
    let (mut rfd, mut reader) = _open_reader(input);
    let (mut wfd, mut writer) = _open_writer(output);
    let mut lines: u32 = 0;
    loop {
        let (_rfd, _reader) = _read(rfd, reader);
        reader = _reader;
        rfd = _rfd.clone();
        if _rfd.eof {
            break;
        }
        wfd.buf = rfd._get_buf();
        let (_wfd, _writer) = _write(wfd.clone(), writer);
        writer = _writer;
        lines += 1;
    }
    let _ = writer.flush();
    lines
}

#[cfg(test)]
#[test]
fn path_test() {
    let path: &Path = Path::new(_TEST);
    let sp: String = path_to_string(path);
    assert_eq!(_TEST, sp);
    println!("ok, Path-->String: {}", sp);
}

const _TEST: &str = "../foo/var";
const _IN: &str = "SAMPLE.html";
const _OUT: &str = "~SAMPLE.html";
const _ERR: &str = "ERROR.html";

pub fn _run() {
    // path_test();
    // _reader_test01(_IN);
    // let lines = _writer_test01(_IN, _OUT);
}

/**
 * ANSI escape code - Colors
 */
const RESET: &str = "\x1b[m"; // [0m
const RED: &str = "\x1b[91m"; // error
const _GREEN: &str = "\x1b[92m"; // debug
const _YELLOW: &str = "\x1b[93m"; // title
const BLUE: &str = "\x1b[94m"; // options
const _MAGENTA: &str = "\x1b[95m"; // warning
const CYAN: &str = "\x1b[96m"; // information

// オーバーロード機能は無いですか? → ジェネリックスを使えば良さげです。
// To RED → msg: は &str or String を受入れる
pub fn red<T: std::fmt::Display>(msg: T) -> String {
    format!("{}{}{}", RED, msg, RESET)
}
// To BLUE
pub fn blue<T: std::fmt::Display>(msg: T) -> String {
    format!("{}{}{}", BLUE, msg, RESET)
}
// To Cyan
pub fn cyan<T: std::fmt::Display>(msg: T) -> String {
    format!("{}{}{}", CYAN, msg, RESET)
}

// Error message → file: は &str or &Path を受入れる
fn error<T: std::fmt::Debug>(msg: &str, file: T, e: String) -> String {
    format!("{}: {:?} {}", red(msg), file, red(e))
}
