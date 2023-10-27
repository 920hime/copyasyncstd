use regex::Regex;
use std::collections::VecDeque;
use std::sync::atomic::AtomicI32;
/**
 * Copyright (C) 2009 awk4j - https://ja.osdn.net/projects/awk4j/
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
use std::sync::Mutex;
use std::time::SystemTime;
// use tokio::sync::Mutex;

use crate::atomic;
use crate::files::DD;
use crate::iomod;

// スレッドセーフな STACK の実装

lazy_static! { // App start time
    // https://ytyaru.hatenablog.com/entry/2020/12/15/000000
    static ref _START_TIME: SystemTime = std::time::SystemTime::now();
}

static _SEQ_NO: AtomicI32 = AtomicI32::new(0); // sequence number
static _REQ_NO: AtomicI32 = AtomicI32::new(0); // current queue number
static _IS_EOF: AtomicI32 = AtomicI32::new(0); // Main process ends

static _PROGRESS: AtomicI32 = AtomicI32::new(0); // progress lock
                                                // static mut PRINT: Mutex<i32> = Mutex::new(0); // print lock
lazy_static! {
    static ref LOCK: Mutex<i32> = Mutex::new(0); // STACK lock
}
static mut STACK: Vec<DD> = Vec::new(); // Vector
static mut QUEUE: VecDeque<DD> = VecDeque::new(); // VecDeque

static mut FIFO: bool = true; // FIFO(QUEUE) FILO(STACK)

// static mut MUTEX: OnceCell<Mutex<i32>> = OnceCell::new();
// static mut ONCE_CELL: OnceCell<Vec<DD>> = OnceCell::new();

// スレッド対応の push - by unsafe Rust power
fn push(dd: DD) -> () {
    unsafe {
        let _lock = LOCK.lock();
        if FIFO {
            QUEUE.push_back(dd) // 後入れ後出し
        } else {
            STACK.push(dd) // 後入れ先出し
        }
    }
}
// スレッド対応の pop - by unsafe Rust power
fn pop() -> Option<DD> {
    unsafe {
        let _lock = LOCK.lock();
        if FIFO {
            QUEUE.pop_front() // 先入れ先出し
        } else {
            STACK.pop() // 先入れ後出し
        }
    }
}

// pop ヘルパー
pub fn get() -> Option<DD> {
    let _pop = pop();
    // match _pop {
    //     Some(x) =>  Some(x),
    //     None => None,
    // }
    _pop
}
// push ヘルパー
pub fn put(dd: DD) {
    // println!("put: {}", dd.input);
    atomic::atomic_add(&_SEQ_NO, 1); // sequence number
    atomic::atomic_add(&_REQ_NO, 1); // current queue number
    push(dd);
}

// キューの終了判定
pub fn is_done() -> bool {
    let eof: bool = atomic::atomic_bool_get(&_IS_EOF);
    let req: i32 = atomic::atomic_get(&_REQ_NO);
    eof && req <= 0
}

// ロックを獲得できなければ (!=1) 処理をスキップする
pub fn progress(input: &String) {
    // 何も表示されなくなる！ ^^);
    // if !atomic::atomic_bool_get_set(&_PROGRESS, true) {
        let seq = atomic::atomic_get(&_SEQ_NO); // sequence number
        let req = atomic::atomic_add(&_REQ_NO, -1); // 処理要求を減算
        let ela = elapsed_time(); // 経過時間
        let cya = iomod::cyan(&format!("{} {}/{}", ela, req, seq));
        let pack: String = pack_path(input);
        // \e[nK カーソルより後ろを消去
        // \e[nA 上にn移動
        // print!("{}: {}\x1b[K\r", cya, dd.input); // NG
        // unsafe {
        // let _print = PRINT.lock().unwrap();
        println!("{}: {}\x1b[K\x1b[1A", cya, pack);
        // }
    // }
    // atomic::atomic_bool_set(&_PROGRESS, false);
}
pub fn progress_fin(message: &str) {
    let seq = atomic::atomic_get(&_SEQ_NO); // sequence number
    let req = atomic::atomic_get(&_REQ_NO); // current queue number
    let ela = elapsed_time(); // 経過時間
    let cya = iomod::cyan(&format!("{} {}/{}", ela, req, seq));
    // unsafe {
    // let _print = PRINT.lock().unwrap();
    println!("{}: {}", cya, message);
    // }
}
// https://ytyaru.hatenablog.com/entry/2020/12/15/000000
// as_millis, as_micros, as_nanos
pub fn elapsed_time() -> String {
    let now: SystemTime = std::time::SystemTime::now();
    if let Ok(epoch) = now.duration_since(*_START_TIME) {
        let hou: u64 = epoch.as_secs() / 3600_u64; // hours >
        let tmp: u64 = epoch.as_secs() % 3600_u64; // minutes
        let min: u64 = tmp / 60_u64; // minutes
        let sec: u64 = tmp % 60_u64; // secs
        let mil: &str = &format!("{:09}", epoch.as_nanos())[..3];
        return format!("{}:{}:{:02}.{}", hou, min, sec, mil);
    }
    "".to_string() // start > now
}

const RE_PACK: &str = r"^[^/]+/";
const PATH_WIDTH: usize = 64; // バイト長

// パスをパック表示する
fn pack_path(path: &str) -> String {
    lazy_static! { // (Regex は一度だけコンパイルされる)
        static ref RE: Regex = Regex::new(RE_PACK).unwrap();
    }
    let mut rs = iomod::path_to_unix(path);
    while &rs.len() > &PATH_WIDTH && RE.is_match(&rs) {
        rs = RE.replace(&rs, "… ").to_string(); // 置換
    }
    rs
}

pub fn initialize(fifo: bool, _capacity: usize) {
    let _ = elapsed_time(); // Initialize start time

    unsafe {
        let _lock = LOCK.lock();
        FIFO = fifo; // First in First out
        if fifo {
            // キャパシティの設定
            QUEUE.reserve_exact(_capacity);
            // println!("QUEUE_CAPACITY: {}", QUEUE.capacity());
        } else {
            STACK.reserve(_capacity);
            // println!("STACK_CAPACITY: {}", STACK.capacity());
        }
    }
}
pub fn terminator() {
    atomic::atomic_bool_set(&_IS_EOF, true); // EOF マークの設定
}

//  https://doc.rust-jp.rs/book-ja/ch19-01-unsafe-rust.html
// unsafe Rustでできること
// ・可変で静的な変数にアクセスしたり変更すること
// ・生ポインタを参照外しすること
// ・unsafeな関数やメソッドを呼ぶこと
// ・unsafeなトレイトを実装すること
/*
Rustにunsafeな分身がある理由は、根本にあるコンピュータのハードウェアが本質的に
unsafeだからです。Rustがunsafeな処理を行わせてくれなかったら、特定の仕事を行えません。
Rustは、低レベルなシステムプログラミングを許可する必要があります。
直接OSと相互作用したり、独自のOSを書くことさえもそうです。
低レベルなシステムプログラミングに取り組むことは、言語の目標の1つなのです。
*/
// 可変で静的な変数を定義
static mut _TAX: f32 = 0.1;
fn _test() {
    unsafe {
        _TAX = 0.08; // use of mutable static
        println!("Price: {}", _TAX * 300.0);
    }
}
