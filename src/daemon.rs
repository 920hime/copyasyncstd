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
use std::sync::atomic::AtomicI32;
use std::time::Duration;

use crate::asyncmod;
use crate::atomic;
use crate::files;
use crate::files::DD;
use crate::thmod;

// RustのTokioで非同期とグリーンスレッドを理解する
// https://zenn.dev/tfutada/articles/5e87d6e7131e8e

// https://doc.rust-jp.rs/book-ja/ch16-03-shared-state.html
use async_std::task::JoinHandle;
static mut HANDLES: Vec<JoinHandle<()>> = Vec::new();
static THREADS: AtomicI32 = AtomicI32::new(0);

pub fn set_threads(threads: i32) {
    atomic::atomic_set(&THREADS, threads); // threads number
}

pub fn main() {
    use async_std::task;
    let _ = task::block_on(spawnx()); // 初期スレッドの起動
}
// グリーンスレッド
async fn spawnx() {
    use async_std::task;
    let threads: i32 = atomic::atomic_get(&THREADS);
    for _ in 0..threads {
        let handle: JoinHandle<()> = task::spawn(async {
            loop {
                let _pop = thmod::get(); // リクエストを取得
                match _pop {
                    Some(x) => task(x).await, // タスクを開始
                    None => {
                        if thmod::is_done() {
                            break;
                        } else {
                            sleep().await;
                        }
                    }
                };
            }
        });
        unsafe {
            HANDLES.push(handle); // スレッドハンドルを登録
        }
        // _joinall().await; // spawn task で使用する
    }
    waiting_for_completion().await; // 完了待ち
}

// スレッドの完了を待ち合わせる (called from main)
pub async fn _joinall() {
    // use futures::future::join;
    unsafe {
        while HANDLES.len() > 0 {
            let _handle: JoinHandle<()> = HANDLES.pop().unwrap();
            // let _ = join(_handle);
        }
    }
}

// 完了待ち
pub async fn waiting_for_completion() {
    while !thmod::is_done() {
        sleep().await;
    }
}

// https://runebook.dev/ja/docs/rust/std/thread/fn.sleep
// https://doc.rust-lang.org/1.64.0/std/thread/fn.sleep.html
//
static _IDLE: AtomicI32 = AtomicI32::new(0); // スリープ時間のもとになるカウンター
static _DELAY: AtomicI32 = AtomicI32::new(0); // スリープする時間

fn reset_sleep() {
    atomic::atomic_set(&_IDLE, 0);
    atomic::atomic_set(&_DELAY, 1);
}

async fn sleep() {
    use async_std::task;
    let mut idle: i32 = atomic::atomic_get(&_IDLE);
    let mut delay: i32 = atomic::atomic_get(&_DELAY);
    if idle > 10_000 {
        idle = 0;
    }
    if idle > 10 {
        // 10回以上何もせずにループしていた場合、
        // 次のループ以降のスリープ時間を2倍ずつ増やしていく
        // このときに最大スリープ時間は10,000マイクロ秒としている(10ミリ秒)
        delay = (delay * 2).min(10_000);
    } else {
        // ループのたびにidelをインクリメントする
        // idelが10に満たないときはスリープ時間は一律で1,000マイクロ秒となる(1ミリ秒)
        idle += 1;
        delay = 1000;
    }
    atomic::atomic_set(&_IDLE, idle);
    atomic::atomic_set(&_DELAY, delay);
    // 指定されたマイクロ秒分だけスリープする
    task::sleep(Duration::from_micros(delay as u64)).await;
    // println!("IDLING: {} {} ", idle, delay);
}

async fn task(dd: DD) -> () {
    // println!("task: {}", dd.input); ////
    reset_sleep();
    let input: &String = &dd.input.clone();
    let output: &String = &dd.output.clone();
    if dd.action != files::SKIP {
        // スキップ以外ならアクションを実行
        if dd.cmr_mode == files::_RENAME {
            asyncmod::rename_file(input, output).await; // Rename
        } else if dd.algorithm == files::_STD {
            asyncmod::copy(input, output).await; // Copy, Move
        } else if dd.algorithm == files::_MAXBUF {
            asyncmod::copymax(input, output).await; // maxbuf
        } else if dd.algorithm == files::_TEST {
            let _ = async_std::fs::copy(input, output).await; // Test
            println!("exit test: {}", input); ////
        } else {
            asyncmod::copych(input, output).await; // channel
        }
    }
    if dd.cmr_mode != files::_COPY {
        // 入力ファイルを削除
        let _ = asyncmod::remove_file(input).await; // Move, Rename
    }
    thmod::progress(input); // プログレス
}
