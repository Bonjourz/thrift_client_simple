use tokio::io::{AsyncRead, AsyncWrite};
use std::io::{self, ErrorKind};
use std::future::{Future};
use std::marker::Unpin;
use std::pin::Pin;
use std::task::{Context, Poll};
use futures::ready;
use bytes::Buf;
use pin_project_lite::pin_project;
use std::mem::size_of;

macro_rules! read_impl {
    (
        $(
            $(#[$outer:meta])*
            fn $name:ident(&mut self) -> $($fut:ident)*;
        )*
    ) => {
        $(
            $(#[$outer])*
            fn $name<'a>(&'a mut self) -> $($fut)*<Self> where Self: Unpin {
                //$($fut)*::new(self)
                $($fut)*::new(self)
            }
        )*
    }
}

macro_rules! reader {
    ($name:ident, $ty:ty, $reader:ident) => {
        reader!($name, $ty, $reader, size_of::<$ty>());
    };
    ($name:ident, $ty:ty, $reader:ident, $bytes:expr) => {
        //pin_project! {
            #[doc(hidden)]
            pub struct $name<'a, R: ?Sized> {
                //#[pin]
                src: &'a mut R,
                pub buf: [u8; $bytes],
                pub read: u8,
            }
        //}

        impl<'a, R> $name<'a, R> {
            pub(crate) fn new(src: &'a mut R) -> Self {
                $name {
                    src,
                    buf: [0; $bytes],
                    read: 0,
                }
            }
        }

        impl<R> Future for $name<'_, R>
        where
            R: AsyncRead + Unpin,
        {
            type Output = io::Result<$ty>;

            fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
                //let mut me = self.project();
				let mut me = &mut *self;

                if me.read == $bytes as u8 {
                    return Poll::Ready(Ok(Buf::$reader(&mut &me.buf[..])));
                }

                while me.read < $bytes as u8 {
                    me.read += match Pin::new(&mut *me
                        .src)
                        //.as_mut()
                        .poll_read(cx, &mut me.buf[me.read as usize..])
                    {
                        Poll::Pending => return Poll::Pending,
                        Poll::Ready(Err(e)) => return Poll::Ready(Err(e.into())),
                        Poll::Ready(Ok(0)) => {
                            return Poll::Ready(Err(ErrorKind::UnexpectedEof.into()));
                        }
                        Poll::Ready(Ok(n)) => n as u8,
                    };
                }

                let num = Buf::$reader(&mut &me.buf[..]);

                Poll::Ready(Ok(num))
            }
        }
    };
}

//////////////////////////// macros end here /////////////////////////

reader!(ReadI32, i32, get_i32);

pub fn read_exact<'a, A>(reader: &'a mut A, buf: &'a mut [u8]) -> ReadExact<'a, A>
where
    A: AsyncRead + Unpin + ?Sized,
{
    ReadExact {
        reader,
        buf,
        pos: 0,
    }
}


pub struct ReadExact<'a, A: ?Sized> {
    reader: &'a mut A,
    buf: &'a mut [u8],
    pub pos: usize,
}

fn eof() -> io::Error {
    io::Error::new(io::ErrorKind::UnexpectedEof, "early eof")
}

impl<A> Future for ReadExact<'_,  A>
where
    A: AsyncRead + Unpin + ?Sized,
{
    type Output = io::Result<usize>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<usize>> {
        loop {
            // if our buffer is empty, then we need to read some data to continue.
            if self.pos < self.buf.len() {
                let me = &mut *self;
                let n = ready!(Pin::new(&mut *me.reader).poll_read(cx, &mut me.buf[me.pos..]))?;
                me.pos += n;
                if n == 0 {
                    return Err(eof()).into();
                }
            }

            if self.pos >= self.buf.len() {
                return Poll::Ready(Ok(self.pos));
            }
        }
    }
}

///////////////////////////// AsyncReadExt ////////////////////////////

pub trait AsyncReadExt: AsyncRead {
        fn read_exact<'a>(&'a mut self, buf: &'a mut [u8]) -> ReadExact<'a, Self>
        //fn read_exact<'a>(mut self: Pin<&mut Self>, buf: &'a mut [u8]) -> ReadExact<'a, Self>
        where
            Self: Unpin,
        {
            read_exact(self, buf)
        }

		//read_impl! {
		//	fn read_i32(&mut self) -> ReadI32;
		//
		//}

		fn read_i32(&mut self) -> ReadI32<Self> 
		where 
			Self: Sized
		{
			ReadI32::new(self)
		}
}

impl<R: AsyncRead + ?Sized> AsyncReadExt for R {}
