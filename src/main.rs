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
#[macro_use]
extern crate lazy_static;
use chrono::{DateTime, Local};
use regex::Regex;
use std::env;
use std::path::Path;

use crate::files::EE;

mod asyncmod;
mod atomic;
mod daemon;
mod files;
mod iomod;
mod thmod;

const INPUT_: &str = "_IN";
const OUTPUT_: &str = "_OUT";

// const _RE_MOUNTPOINT: &str = r"^(/[^/]+)//"; // UNIX 形式
const RE_DRIVE: &str = r"^//[?.]/([a-zA-Z]:)/"; // UNC 形式
/**
 * path -> absolute_path(canonicalize) -> ドライブ名
 *
 * Windows上では絶対パスは「\\?\D:\foo」のような「UNC path」を返す
 * このパスからドライブ名「D:」を抽出して返す
 */
fn get_drive(path: &Path) -> String {
    lazy_static! {
        static ref RE_DV: Regex = Regex::new(RE_DRIVE).unwrap();
    }
    let abs_in = iomod::absolute_path(path).replace(r"\", r"/");
    let _m = RE_DV.captures(&abs_in);
    match _m {
        Some(caps) => {
            caps[1].to_string() // drive name
        }
        None => String::new(), // empty
    }
}

const RE_CAPA: &str = r"^[+]\d+$"; // キャパシティ
const RE_THREAD: &str = r"^[-]\d+$"; // スレッド数
/**
 * Initialize - Command line parameter analysis
 */
fn initialize() -> (String, String, EE, i32) {
    lazy_static! { // (Regex は一度だけコンパイルされる)
        static ref RE_CA: Regex = Regex::new(RE_CAPA).unwrap();
        static ref RE_TH: Regex = Regex::new(RE_THREAD).unwrap();
    }
    let args: Vec<String> = env::args().collect();
    let len = args.len();
    if len < 3 {
        let message = iomod::red("入出力フォルダが省略されています");
        eprintln!("{}: {:?}", message, args);
    }

    let _input: &str = if len > 1 { &args[1] } else { INPUT_ };
    let _output: &str = if len > 2 { &args[2] } else { OUTPUT_ };
    let __input: String = iomod::path_to_unix(_input);
    let __output: String = iomod::path_to_unix(_output);
    let input: &Path = Path::new(_input); // 入力フォルダ
    let output: &Path = Path::new(_output); // 出力フォルダ
    iomod::mkdir(input);
    iomod::mkdir(output);

    let mut queue: &str = "-q[ueue]";
    let mut fifo: bool = true;
    let mut cmr_name: &str = "-c[opy]";
    let mut cmr_mode: char = files::_COPY;
    let mut threads: i32 = 3;
    let mut capa: usize = 2048;
    let mut algorithm: u8 = files::_STD;
    let mut algoname = "std";
    for i in 3..len {
        let argi = &args[i];
        if argi.starts_with("-") {
            if argi == "-c" {
                cmr_name = "-c[opy]";
                cmr_mode = files::_COPY;
            } else if argi == "-m" {
                cmr_name = "-m[ove]";
                cmr_mode = files::_MOVE;
            } else if argi == "-r" {
                cmr_name = "-r[ename]";
                cmr_mode = files::_RENAME;
            } else if argi == "-q" {
                queue = "-q[ueue]";
                fifo = true;
            } else if argi == "-s" {
                queue = "-s[tack]";
                fifo = false;
            } else if RE_TH.is_match(argi) {
                let tmp: i32 = argi.parse().unwrap();
                if tmp != 0 {
                    threads = tmp.abs();
                }
            } else {
                let message = iomod::red("オプションエラー");
                eprintln!("{}: {:?}", message, argi);
            }
        } else if RE_CA.is_match(argi) {
            capa = argi.parse().unwrap();
        } else if argi.starts_with("m") {
            algorithm = files::_MAXBUF;
            algoname = "maxbuf";
        } else if argi.starts_with("c") {
            algorithm = files::_CHANNEL;
            algoname = "channel";
        } else if argi.starts_with("t") {
            algorithm = files::_TEST;
            algoname = "test";
        }
    }
    if !input.is_dir() {
        let message = iomod::red("フォルダではありません".to_string());
        panic!("{}: {:?}", message, __input);
    }
    if !output.is_dir() {
        let message = iomod::red("フォルダではありません".to_string());
        panic!("{}: {:?}", message, __output);
    }
    // \\?\D:\foo
    // Windowsは「\\?\」で始まるパスは解釈処理をせず、そのまま扱う
    let i_drv: String = get_drive(input);
    let o_drv: String = get_drive(output);

    let _start_time: DateTime<Local> = Local::now();
    println!("{}", _start_time);
    println!(" {}: [{}] {}", iomod::blue("Input Folder"), i_drv, __input);
    println!("{}: [{}] {}", iomod::blue("Output Folder"), o_drv, __output);
    print!("{}: {}, ", iomod::blue("Mode"), cmr_name);
    print!("{}: {}, ", iomod::blue("Queue"), queue);
    print!("{}: -{}, ", iomod::blue("Threads"), threads);
    print!("{}: +{}, ", iomod::blue("Capacity"), capa);
    println!("{}: {}", iomod::blue("Algorithm"), algoname);
    if cmr_mode == files::_RENAME && i_drv != o_drv {
        let message = iomod::red("別のドライブには移動できません".to_string());
        panic!("{}", message);
    }
    let ee = EE {
        cmr_mode,  // copy, move, rename
        algorithm, // algorithm
    };
    thmod::initialize(fifo, capa);
    (_input.to_string(), _output.to_string(), ee, threads)
}

/**
 * main
 */
fn main() {
    let (input, output, ee, threads) = initialize();
    let _ = files::search_fils(&input, &output, ee); // リクエストを投げる
    thmod::terminator();
    daemon::set_threads(threads);
    daemon::main(); // スレッド起動
    // async {
    //     daemon::waiting_for_completion().await; // 完了待ち
    // };
    // async {
    //     daemon::joinall().await;
    // };
    println!();
    if ee.cmr_mode != files::_COPY {
        // 移動済みの入力フォルダをファイルを含めてまるごと削除
        iomod::remove_dir_all(&input);
    }
    thmod::progress_fin("Finished");
    run();
}

// Test
fn run() {
    atomic::_run();
    // iomod::_run();
}

/*
Rust には2つの文字列型があります。
&str - 文字列スライスとも呼ばれるプリミティブな文字列型。
String - 標準ライブラリの提供する文字列型。文字列操作などに使う。
ざっくり理解すると借用が &str で、所有権があるのが String と覚えておくとよさげです。
*/
fn _strings() {
    // 文字列リテラルは &str
    let _a: &str = "hello";

    // String -> &str
    let _b: &str = String::from("hello").as_str();

    // String の初期化
    let _c: String = String::from("hello");

    // &str -> String
    let _d: String = "hello".to_string();

    // String -> &str
    let _e: &str = _c.as_str();

    // String -> &str
    let _e: &str = &_c;

    // Mutable variable
    let mut _d: String = String::new();
    _d += "ABC";
    _d += "def";
    println!("mut String {:?}", _d);
    assert_eq!(_d, "ABCdef");

    let _ch1: char = _a.chars().nth(0).unwrap();
    let _ch2: &str = &_a[..3];

    // .contains()
    if _a.find("l").is_some() {
        println!("'l' is in the string!");
    }
}

/*
main:   200
atomic:  60
daemon:  90
files:  100
iomod:  340
thmod:  200
Total:  990
*/
