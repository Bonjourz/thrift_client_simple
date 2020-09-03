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

pin_project! {
	pub struct TAsyncSimpleReadTransport<C>
	where
		C: AsyncRead,
	{
		#[pin]
		chan: C,
	}
}

//impl<C> Read for TAsyncSimpleReadTransport<C>
//where
//    C: AsyncRead,
//{
//    fn read(&mut self, buf: &mut [u8]) 
//		-> Result<usize> {
//			//println!("simple transport read");
//		self.chan.read(buf)
//	}
//}

impl<C> TAsyncSimpleReadTransport<C>
where
    C: AsyncRead,
{
    pub fn new(channel: C) -> TAsyncSimpleReadTransport<C> {
        TAsyncSimpleReadTransport {
			chan: channel,
        }
    }

}

impl<C> AsyncRead for TAsyncSimpleReadTransport<C>
where
    C: AsyncRead,
{
	fn poll_read(
			self: Pin<&mut Self>, 
			cx: &mut Context<'_>, 
			buf: &mut [u8],
	) -> Poll<io::Result<usize>> {
		println!("simple poll read");
		let mut me = self.project(); 
		me.chan.poll_read(cx, buf)
	}
}

/// Factory for creating instances of `TAsyncSimpleReadTransport`.
#[derive(Default)]
pub struct TAsyncSimpleReadTransportFactory;

impl TAsyncSimpleReadTransportFactory {
    pub fn new() -> TAsyncSimpleReadTransportFactory {
        TAsyncSimpleReadTransportFactory {}
    }
}

impl TAsyncReadTransportFactory for TAsyncSimpleReadTransportFactory {
    fn create(&self, channel: Box<dyn AsyncRead + Send + Unpin>) -> Box<dyn TAsyncReadTransport + Send + Unpin> {
        Box::new(TAsyncSimpleReadTransport::new(channel))
    }
}

/////////////////// Write transport start here ///////////////////////

pin_project! {
	pub struct TAsyncSimpleWriteTransport<C>
	where
		C: AsyncWrite,
	{
		#[pin]
		chan: C,
	}
}

impl<C> TAsyncSimpleWriteTransport<C>
where
    C: AsyncWrite,
{
    pub fn new(channel: C) -> TAsyncSimpleWriteTransport<C> {
        TAsyncSimpleWriteTransport {
            chan: channel,
        }
    }

}

//impl<C> Write for TAsyncSimpleWriteTransport<C>
//where
//    C: AsyncWrite,
//{
//    fn write(&mut self, buf: &[u8]) 
//		-> Result<usize> {
//		self.chan.write(buf)
//	}
//
//    fn flush(&mut self) 
//		-> Result<()> {
//		self.chan.flush()
//	}
//
//}

impl<C> AsyncWrite for TAsyncSimpleWriteTransport<C>
where
    C: AsyncWrite,
{
	fn poll_write(
			self: Pin<&mut Self>, 
			cx: &mut Context<'_>, 
			buf: &[u8],
	) -> Poll<io::Result<usize>> {
		println!("simple poll write");
		let mut me = self.project(); 
		me.chan.poll_write(cx, buf)
	}
	
	fn poll_flush(
			self: Pin<&mut Self>, 
			cx: &mut Context<'_>, 
	) -> Poll<io::Result<()>> {
		//todo!()
		println!("simple poll flush");
		let mut me = self.project(); 
		me.chan.poll_flush(cx)
	}
	
	fn poll_shutdown(
			self: Pin<&mut Self>, 
			cx: &mut Context<'_>, 
	) -> Poll<io::Result<()>> {
		//todo!()
		println!("simple poll shutdown");
		let mut me = self.project(); 
		me.chan.poll_shutdown(cx)
	}
}

/// Factory for creating instances of `TAsyncSimpleWriteTransport`.
#[derive(Default)]
pub struct TAsyncSimpleWriteTransportFactory;

impl TAsyncSimpleWriteTransportFactory {
    pub fn new() -> TAsyncSimpleWriteTransportFactory {
        TAsyncSimpleWriteTransportFactory {}
    }
}

impl TAsyncWriteTransportFactory for TAsyncSimpleWriteTransportFactory {
    fn create(&self, channel: Box<dyn AsyncWrite + Send + Unpin>) -> Box<dyn TAsyncWriteTransport + Send + Unpin> {
        Box::new(TAsyncSimpleWriteTransport::new(channel))
    }
}

