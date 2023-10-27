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
// https://doc.rust-lang.org/std/sync/atomic/
// https://runebook.dev/ja/docs/rust/std/sync/atomic/struct.atomici32
use std::sync::atomic::{AtomicI32, Ordering};

/**
 * AtomicInt の実装
 */
// 演算結果を返す (オーバーロードは無いようだ！)
pub fn atomic_get(a: &AtomicI32) -> i32 {
    let x = (*a).load(Ordering::SeqCst);
    x
}
pub fn atomic_set(a: &AtomicI32, n: i32) -> i32 {
    (*a).store(n, Ordering::SeqCst);
    n
}
pub fn atomic_add(a: &AtomicI32, n: i32) -> i32 {
    let x = (*a).fetch_add(n, Ordering::SeqCst); // 加算前の値が帰る
    x + n
}
/**
 * AtomicBool での実装をあきらめた ^^);
 */
pub fn atomic_bool_get(a: &AtomicI32) -> bool {
    let x = (*a).load(Ordering::SeqCst);
    x != 0
}
pub fn atomic_bool_set(a: &AtomicI32, b: bool) -> bool {
    let x: i32 = if b { 1 } else { 0 };
    (*a).store(x, Ordering::SeqCst);
    b
}
pub fn atomic_bool_get_set(a: &AtomicI32, b: bool) -> bool {
    let r = (*a).load(Ordering::SeqCst);
    let x: i32 = if b { 1 } else { 0 };
    (*a).store(x, Ordering::SeqCst);
    r != 0 // 変更前の値を返す
}

static _COUNTER: AtomicI32 = AtomicI32::new(-88); // Test
static _BOOL: AtomicI32 = AtomicI32::new(0);

#[cfg(test)]
#[test]
fn test() {
    atomic_set(&_COUNTER, 0);
    assert_eq!(1, atomic_add(&_COUNTER, 1));
    assert_eq!(2, atomic_add(&_COUNTER, 1));
    assert_eq!(2, atomic_get(&_COUNTER));

    assert_eq!(true, atomic_bool_set(&_BOOL, true));
    assert_eq!(true, atomic_bool_get(&_BOOL));
}

pub fn _run() {
    atomic_set(&_COUNTER, 0);
    assert_eq!(false, atomic_bool_get_set(&_BOOL, true));
}
