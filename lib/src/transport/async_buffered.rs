// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements. See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership. The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License. You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. See the License for the
// specific language governing permissions and limitations
// under the License.

//use std::io::{self, Read, Write, Error, Result};
//use futures::{Poll, Async};
use std::io;
use tokio::io::{AsyncRead, AsyncWrite};
//use bytes::{Buf, BufMut};
use std::task::{Poll, Context};

use super::{TAsyncReadTransport, TAsyncReadTransportFactory, TAsyncWriteTransport, TAsyncWriteTransportFactory};
use std::pin::Pin;
use pin_project_lite::pin_project;

use std::cmp;
use futures::ready;

//const READ_CAPACITY: usize = 4096;
const READ_CAPACITY: usize = 4096;
use std::slice;
//const WRITE_CAPACITY: usize = 4096;
const WRITE_CAPACITY: usize = 4096;


pin_project! {
	pub struct TAsyncBufferedReadTransport<C>
	where
		C: AsyncRead,
	{
		#[pin]
		chan: C,
		buf: Box<[u8]>,
		pos: usize,
		cap: usize,
	}
}

impl<C> TAsyncBufferedReadTransport<C>
where
    C: AsyncRead + Unpin,
{

    pub fn new(channel: C) -> TAsyncBufferedReadTransport<C> {
        TAsyncBufferedReadTransport::with_capacity(READ_CAPACITY, channel)
    }

    pub fn with_capacity(read_capacity: usize, channel: C) -> TAsyncBufferedReadTransport<C> {
        TAsyncBufferedReadTransport {
            buf: vec![0; read_capacity].into_boxed_slice(),
            pos: 0,
            cap: 0,
            chan: channel,
        }
    }

    fn get_bytes(&mut self,
			cx: &mut Context<'_>, 
			) -> Poll<io::Result<&[u8]>> {
        if self.cap - self.pos == 0 {
            self.pos = 0;
            self.cap = ready!(Pin::new(&mut self.chan).poll_read(cx, &mut self.buf))?;
        }

		Poll::Ready(Ok(&self.buf[self.pos..self.cap]))
    }

    fn consume(&mut self, consumed: usize) {
        self.pos = cmp::min(self.cap, self.pos + consumed);
    }
}

impl<C> AsyncRead for TAsyncBufferedReadTransport<C>
where
    C: AsyncRead + Unpin,
{
	fn poll_read(
			self: Pin<&mut Self>, 
			cx: &mut Context<'_>, 
			buf: &mut [u8],
	) -> Poll<io::Result<usize>> {
		//println!("Buffered poll read");
		let mut me = self.project(); 

		let mut bytes_read = 0;
		
		//loop {
			let nread = {
				if *me.cap - *me.pos == 0 {
					*me.cap = ready!(Pin::new(&mut *me.chan).poll_read(cx, me.buf))?;
					*me.pos = 0;
					//println!("need poll_read, cap {}", me.cap);
				}
				let avail_bytes = *me.cap - *me.pos;
				
				let avail_space = buf.len() - bytes_read;
				let nread = cmp::min(avail_space, avail_bytes);
				buf[bytes_read..(bytes_read + nread)].copy_from_slice(&me.buf[*me.pos..(*me.pos + nread)]);
				nread
			};

			*me.pos = cmp::min(*me.cap, *me.pos + nread);
			//println!("cap: {}, pos: {}", *me.cap, *me.pos);
			bytes_read += nread;
			//println!("nread {}, bytes_read {}, len {}", 
			//		nread, bytes_read, buf.len());

			//if bytes_read == buf.len() || nread == 0 {
			//	break;
			//}
		//}
		Poll::Ready(Ok(bytes_read))
	}
}

/// Factory for creating instances of `TAsyncBufferedReadTransport`.
#[derive(Default)]
pub struct TAsyncBufferedReadTransportFactory;

impl TAsyncBufferedReadTransportFactory {
    pub fn new() -> TAsyncBufferedReadTransportFactory {
        TAsyncBufferedReadTransportFactory {}
    }
}

impl TAsyncReadTransportFactory for TAsyncBufferedReadTransportFactory {
    fn create(&self, channel: Box<dyn AsyncRead + Send + Unpin>) -> Box<dyn TAsyncReadTransport + Send + Unpin> {
        Box::new(TAsyncBufferedReadTransport::new(channel))
    }
}

/////////////////// Write transport start here ///////////////////////

pin_project! {
	pub struct TAsyncBufferedWriteTransport<C>
	where
		C: AsyncWrite,
	{
		#[pin]
		chan: C,
		buf: Vec<u8>,
		cap: usize,
		pos: usize,
	}
}

impl<C> TAsyncBufferedWriteTransport<C>
where
    C: AsyncWrite,
{
    pub fn new(channel: C) -> TAsyncBufferedWriteTransport<C> {
        TAsyncBufferedWriteTransport::with_capacity(WRITE_CAPACITY, channel)
    }

    pub fn with_capacity(write_capacity: usize, channel: C) -> TAsyncBufferedWriteTransport<C> {
        assert!(
            write_capacity > 0,
            "write buffer size must be a positive integer"
        );

        TAsyncBufferedWriteTransport {
            buf: Vec::with_capacity(write_capacity),
            cap: write_capacity,
            chan: channel,
			pos: 0,
        }
    }

}

impl<C> AsyncWrite for TAsyncBufferedWriteTransport<C>
where
    C: AsyncWrite + Unpin,
{
	fn poll_write(
			self: Pin<&mut Self>, 
			cx: &mut Context<'_>, 
			buf: &[u8],
	) -> Poll<io::Result<usize>> {
		// println!("Buffered poll write");
		let mut me = self.project(); 
		if !buf.is_empty() {
			let mut avail_bytes;

			loop {
				avail_bytes = cmp::min(buf.len(), *me.cap - me.buf.len());
				// println!("len: {}, cap: {}, melen: {}", 
				// 		buf.len(), *me.cap, me.buf.len());
				if avail_bytes == 0 {
					// println!("ready to write all");
					let mut need_write = me.buf.len() - *me.pos; 
					if need_write > 0 {
						loop {
							let nwrite = ready!(Pin::new(&mut *me.chan).poll_write(cx, &me.buf[*me.pos..]))?;
							*me.pos += nwrite;

							if *me.pos == me.buf.len() {
								// all has been write
								// println!("clear buf in write");
								me.buf.clear();
								*me.pos = 0;
								break;
							}
						}
					}
					ready!(Pin::new(&mut *me.chan).poll_flush(cx))?;
				} else {
					break;
				}
			}

			let avail_bytes = avail_bytes;

			me.buf.extend_from_slice(&buf[..avail_bytes]);
			assert!(me.buf.len() <= *me.cap, "copy overflowed buffer");
			//println!("write {} bytes", avail_bytes);
			Poll::Ready(Ok(avail_bytes))
		} else {
			Poll::Ready(Ok(0))
		}
	}
	
	fn poll_flush(
			self: Pin<&mut Self>, 
			cx: &mut Context<'_>, 
	) -> Poll<io::Result<()>> {
		let mut me = self.project(); 
		// println!("Buffered poll flush, buf len: {}", me.buf.len());
		let mut need_write = me.buf.len() - *me.pos; 
		if need_write > 0 {
			loop {
				let nwrite = ready!(Pin::new(&mut *me.chan).poll_write(cx, &me.buf[*me.pos..]))?;
				*me.pos += nwrite;

				if *me.pos == me.buf.len() {
					// all has been write
					me.buf.clear();
					*me.pos = 0;
					break;
				}
			}
		}
		Pin::new(&mut *me.chan).poll_flush(cx)
	}
	
	fn poll_shutdown(
			self: Pin<&mut Self>, 
			cx: &mut Context<'_>, 
	) -> Poll<io::Result<()>> {
		// println!("Buffered poll shutdown");
		let mut me = self.project(); 
		me.chan.poll_shutdown(cx)
	}
}

/// Factory for creating instances of `TAsyncBufferedWriteTransport`.
#[derive(Default)]
pub struct TAsyncBufferedWriteTransportFactory;

impl TAsyncBufferedWriteTransportFactory {
    pub fn new() -> TAsyncBufferedWriteTransportFactory {
        TAsyncBufferedWriteTransportFactory {}
    }
}

impl TAsyncWriteTransportFactory for TAsyncBufferedWriteTransportFactory {
    fn create(&self, channel: Box<dyn AsyncWrite + Send + Unpin>) -> Box<dyn TAsyncWriteTransport + Send + Unpin> {
        Box::new(TAsyncBufferedWriteTransport::new(channel))
    }
}

