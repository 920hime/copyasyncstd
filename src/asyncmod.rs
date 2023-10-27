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
// https://docs.rs/async-std/latest/async_std/io/trait.WriteExt.html#method.write_all
use crate::Path;
use async_std::fs;
use async_std::fs::File;
use async_std::io::{ReadExt, WriteExt};
use futures::prelude::*;
use futures::StreamExt;

// #[derive(Debug, Clone)] // I/O buffer
struct IOBUF {
    buf: Vec<u8>, // this field does not implement `Copy`
    length: usize,
}
impl IOBUF {
    // fn to_slice(&self) -> &[u8] {
    //     &self.buf[..self.length]
    // }
}

// https://docs.rs/tokio/1.33.0/tokio/fs/fn.copy.html
/**
* copy from to -> length
*
* async fn copy(from: AsRef<Path>, to: AsRef<Path>) -> Result<u64, Error>
*/
pub async fn copy<P: AsRef<Path> + std::convert::AsRef<async_std::path::Path>>(
    from: P,
    to: P,
) -> u64 {
    let length = fs::copy(from, to).await;
    length.unwrap()
}

/**
 * copy maxbuf from to -> length
 */
pub async fn copymax<
    P: AsRef<Path> + std::convert::AsRef<async_std::path::Path> + std::marker::Copy,
>(
    from: P,
    to: P,
) -> u64 {
    const BUFSIZE: usize = 1024 * 1024;
    let mut fr = File::open(from).await.unwrap();
    let mut fw = File::create(to).await.unwrap();
    let mut result: usize = 0;
    let mut io = IOBUF {
        buf: vec![0_u8; BUFSIZE],
        length: 0,
    };
    loop {
        io.length = ReadExt::read(&mut fr, &mut io.buf).await.unwrap();
        if io.length == 0 {
            break;
        }
        let _ = WriteExt::write_all(&mut fw, &io.buf[..io.length]).await;
        result += io.length;
    }
    let _ = WriteExt::flush(&mut fw);
    let fromsize: usize = get_meta_len(&from).await.try_into().unwrap();
    assert_eq!(fromsize, result, "(original:result) ");
    result as u64
}

/**
 * copy channel from | to -> length
 */
pub async fn copych<P: AsRef<Path> + std::convert::AsRef<async_std::path::Path>>(
    from: P,
    to: P,
) -> u64 {
    use async_std::task;
    use async_std::task::JoinHandle;
    use futures::channel::mpsc;
    const BUFSIZE: usize = 1024 * 1024;
    let fromsize: usize = get_meta_len(&from).await.try_into().unwrap();
    let mut fr = File::open(from).await.unwrap();
    let mut fw = File::create(to).await.unwrap();
    let (mut tx, mut rx) = mpsc::channel(4);
    let _handle: JoinHandle<()> = task::spawn(async move {
        loop {
            let mut io = IOBUF {
                buf: vec![0_u8; BUFSIZE],
                length: 0,
            };
            io.length = ReadExt::read(&mut fr, &mut io.buf).await.unwrap();
            if io.length == 0 {
                break;
            }
            let _ = tx.send(io).await;
        }
    });
    // drop(tx);
    let mut result: usize = 0; // 受信
    while let Some(received) = rx.next().await {
        result += received.length; // write の前に使用する
        let _ = WriteExt::write_all(&mut fw, &received.buf[..received.length]).await;
    }
    let _ = WriteExt::flush(&mut fw);
    assert_eq!(fromsize, result, "(original:result) ");
    result as u64
}

/**
 * rename from to - rename 関数を使用した 爆速 move の実装
 *
 * async fn rename(from: impl AsRef<Path>, to: impl AsRef<Path>) -> Result<()>
 */
pub async fn rename_file<P: AsRef<Path> + std::convert::AsRef<async_std::path::Path>>(
    from: P,
    to: P,
) -> () {
    let rs = fs::rename(from, to).await;
    rs.unwrap();
}

/**
 * remove file
 *
 * async fn remove_file(path: impl AsRef<Path>) -> Result<()>
 */
pub async fn remove_file<P: AsRef<Path>>(path: P) -> () {
    let p: &Path = path.as_ref();
    if p.is_file() {
        let _ = fs::remove_file(p).await;
    }
    ()
}

// https://runebook.dev/ja/docs/rust/std/fs/struct.metadata
/**
 * get metadata - length    
 *
 * async fn metadata(path: impl AsRef<Path>) -> Result<Metadata>
 */
pub async fn get_meta_len<P: AsRef<Path> + std::convert::AsRef<async_std::path::Path>>(
    path: P,
) -> u64 {
    let rs = fs::metadata(path).await;
    let metadata: u64 = rs.unwrap().len();
    metadata
}

/* write n
    let mut pos = 0;
    while pos < buf.length {
        let bytes_written = file .write(&buf.buf[pos..]).await?;
        pos += bytes_written;
    }
*/
