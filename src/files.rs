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
use std::io;
use std::path::Path;
// use std::path::{Path, PathBuf};

use crate::iomod;
use crate::thmod;

pub fn search_fils(input: &str, output: &str, ee: EE) -> io::Result<()> {
    let ipath: &Path = Path::new(input);
    let opath: &Path = Path::new(output);
    visit_dir(ipath, opath, ee)?;
    Ok(())
}

fn visit_dir<P: AsRef<Path>>(path: P, opath: &Path, ee: EE) -> io::Result<()> {
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let _name: String = iomod::get_filename(&entry.path());
        let _opath: &Path = &opath.join(_name); // output file
        if entry.file_type()?.is_dir() {
            // println!("+{:?}\t{:?}", entry.path(), _opath);
            iomod::mkdir(_opath); // Create deep folder
            visit_dir(entry.path(), _opath, ee.clone())?;
        } else {
            // println!(".{:?}\t{:?}", entry.path(), _opath);
            make_dd(&entry.path(), _opath, ee.clone());
        }
    }
    Ok(())
}

// https://runebook.dev/ja/docs/rust/std/fs/struct.metadata
// リクエスト(構造体)を作成し投げる
fn make_dd(_input: &Path, _output: &Path, ee: EE) {
    let input: String = iomod::path_to_string(_input);
    let output: String = iomod::path_to_string(_output);
    let action: i8 = judgment(_input, _output);
    let dd = DD {
        input,                   // input file
        output,                  // output file
        action,                  // DO, SKIP
        cmr_mode: ee.cmr_mode,   // copy, move, rename
        algorithm: ee.algorithm, // Buffer number
    };
    thmod::put(dd);
}

// コピーするかどうかを決定する
fn judgment(input: &Path, output: &Path) -> i8 {
    if !output.is_file() {
        return DO; // 出力ファイルが存在しない
    }
    let ilen = iomod::get_meta_len(input);
    let olen = iomod::get_meta_len(output);
    if ilen != olen {
        return DO; // 長さが異なる
    }
    let itime = iomod::get_meta_modified(input);
    let otime = iomod::get_meta_modified(output);
    if let Ok(epoch) = itime.duration_since(otime) {
        if epoch.as_secs() == 0 && epoch.as_millis() == 0 {
            // println!("same = {}.{:03}", epoch.as_secs(), epoch.as_millis());
            return SKIP; // 日時が等しい
        }
    } else {
        return SKIP; // 出力側の日時が新しい
    }
    DO
}

// Action - execution mode (cmr)
pub const _COPY: char = 'c';
pub const _MOVE: char = 'm';
pub const _RENAME: char = 'r';
// Action - Possibility of execution
pub const DO: i8 = 1;
pub const SKIP: i8 = 2;
// Algorithm
pub const _STD: u8 = 0;
pub const _MAXBUF: u8 = 1;
pub const _CHANNEL: u8 = 2;
pub const _TEST: u8 = 9;

// Daemon descriptor - 構造体、クローン可能
#[derive(Debug, Clone)] // String は Copy を実装できない
pub struct DD {
    pub input: String,  // input file
    pub output: String, // output file
    pub action: i8,     // DO, SKIP
    pub cmr_mode: char, // copy, move, rename
    pub algorithm: u8,  // Algorithm
}
impl DD {
    // pub fn _get_input(&self) -> String {
    //     self.input.clone()
    // }
}
#[derive(Debug, Clone, Copy)] // main が作成する DD のサブセット
pub struct EE {
    pub cmr_mode: char, // copy, move, rename
    pub algorithm: u8,  // Algorithm
}
