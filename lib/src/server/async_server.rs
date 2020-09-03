use std::boxed::Box;
use std::io;
use std::net::SocketAddr;
use std::sync::{Arc};
use std::thread;
use std::task::Context;
use std::pin::Pin;

use crate::protocol::{TAsyncBinaryInputProtocol, TAsyncBinaryOutputProtocol
    //TAsyncInputProtocolFactory, TAsyncOutputProtocolFactory, TAsyncInputProtocol, TAsyncOutputProtocol,
};
// use crate::transport::{TAsyncReadTransportFactory, TAsyncWriteTransportFactory};
use crate::transport::{TAsyncBufferedReadTransport, TAsyncBufferedWriteTransport};

use super::TAsyncProcessor;

use futures::{Future};
use std::task::Poll;
use net2;

use tokio::net::{TcpListener, TcpStream};
use tokio::stream::StreamExt;
//use tokio::io::{AsyncRead, AsyncWrite};
use crate::TransportErrorKind;

use pin_project_lite::pin_project;

pub struct TAsyncServer<PRC>
where
    PRC: TAsyncProcessor + Send + Sync + 'static,
{
    processor: Arc<PRC>,
    workers: usize,
    addr: SocketAddr,
}
//
impl<PRC> TAsyncServer<PRC>
where
    PRC: TAsyncProcessor + Send + Sync + 'static,
{
   pub fn new(
       processor: PRC,
       addr: SocketAddr,
       workers: usize,
    ) -> TAsyncServer<PRC> {
       TAsyncServer {
           processor: Arc::new(processor),
           workers: workers,
           addr: addr,
        }
    }

    pub async fn serve(&self) {
        let processor = self.processor.clone();
        let addr = self.addr;
        let workers = self.workers;

        // let threads = (0..self.workers - 1)
        //     .map(|i| {
        //         let processor = processor.clone();

        //         thread::Builder::new()
        //             .name(format!("Worker{}", i))
        //             .spawn(move || {
        //                 println!("call worker func2 wokers");
        //                 worker_func(
        //                    processor, &addr, workers,
        //                 );
        //             })
        //             .unwrap()
        //     })
        //     .collect::<Vec<_>>();
        worker_func(
           processor, &addr, workers,
        ).await;

        // for thread in threads {
        //     thread.join().unwrap();
        // }
   }
}

async fn worker_func<PRC>(
    processor: Arc<PRC>,
    addr: &SocketAddr,
    workers: usize,
) where
    PRC: TAsyncProcessor + Send + Sync + 'static,
{
    println!("a new worker thread");
    let listener = match *addr {
        SocketAddr::V4(_) => net2::TcpBuilder::new_v4().unwrap(),
        SocketAddr::V6(_) => net2::TcpBuilder::new_v6().unwrap(),
    };
    configure_tcp(workers, &listener).unwrap();
    listener.reuse_address(true).unwrap();
    listener.bind(addr).unwrap();

    let mut server = listener
        .listen(1024)
        .and_then(|l| TcpListener::from_std(l))
        .unwrap();

    let mut incoming = server.incoming();

	loop {
		//let Some(stream)= server.await;
		//let Some(stream)= incoming.next().await;
		let stream = match incoming.next().await {
			Some(s) => s,
			None => {continue;}
		};
		//let stream = server.poll_accept(&context);
		let socket = match stream {
			Ok(t) => t,
			Err(e) => {
				println!("failed to connect; err = {:?}", e);
				continue
			}
		};
			
        //println!("message received!");

        {
            let processor = Arc::clone(&processor);
            tokio::spawn(async move {
                let (r_chan, w_chan) = socket.into_split();
                   
                let r_tran = TAsyncBufferedReadTransport::new(r_chan);
                let w_tran = TAsyncBufferedWriteTransport::new(w_chan);
                let mut i_prot = TAsyncBinaryInputProtocol::new(r_tran, false);
                let mut o_prot = TAsyncBinaryOutputProtocol::new(w_tran, false);

                loop {
                    match processor.
                        process(&mut i_prot, 
                        &mut o_prot).await {
                        Ok(_ok) => { /* println!("[gbd] async_server arrive here 1"); */ },
                        Err(_e) => {
                            match _e {
                                crate::Error::Transport(ref transport_err)
                                if transport_err.kind == TransportErrorKind::EndOfFile => {
                                    /*println!("end of file")*/
                                },
                                other => println!("processor completed with error: {:?}", other),
                            };
                            break;
                        },
                    };
                }
            });
        }
	}
}

#[cfg(unix)]
fn configure_tcp(workers: usize, tcp: &net2::TcpBuilder) -> io::Result<()> {
    use net2::unix::*;
    if workers > 1 {
        tcp.reuse_port(true)?;
    }

    Ok(())
}

#[cfg(windows)]
fn configure_tcp(workers: usize, tcp: &net2::TcpBuilder) -> io::Result<()> {
    Ok(())
}

// pin_project! {
// 	pub struct ProcessorWrapper<PRC>
// 	where
// 	    //PRC: TAsyncProcessor + Send + Sync + 'static,
// 	    PRC: TAsyncProcessor,
// 	{
// 		#[pin]
// 	    processor: Arc<PRC>,
// 	    //i_prot: Arc<Mutex<Box<dyn TAsyncInputProtocol + Send>>>,
// 	    //o_prot: Arc<Mutex<Box<dyn TAsyncOutputProtocol + Send>>>,
// 		i_prot: Box<dyn TAsyncInputProtocol + Send>,
// 		o_prot: Box<dyn TAsyncOutputProtocol + Send>,
// 	}
// }

// pub fn procWrap<PRC, RTF, WTF, IPF, OPF>(
// 	processor: Arc<PRC>, 
//    //socket: &mut TcpStream,
//    mut socket: Box<TcpStream>,
//     r_chan: Box<dyn AsyncRead + Send>, 
//     w_chan: Box<dyn AsyncWrite + Send>,
//     r_chan: &mut (dyn AsyncRead + Send + Unpin + Sync + 'a), 
//     w_chan: &mut (dyn AsyncWrite + Send + Unpin + Sync + 'a),
//     r_trans_factory: Arc<RTF>, w_trans_factory: Arc<WTF>,
//     i_proto_factory: Arc<IPF>, o_proto_factory: Arc<OPF>) -> ProcessorWrapper<PRC>
// where
//    PRC: TAsyncProcessor + Send + Sync + 'static,
//    //R: AsyncRead + Unpin + ?Sized + Send,
// 	//W: AsyncWrite + Unpin + ?Sized + Send,
// 	//IP: TAsyncInputProtocol + Send,
//    //OP: TAsyncOutputProtocol + Send,
// { 
// 	//let mut socket = socket.take();
// 	let (r_chan, w_chan) = socket.into_split();
   
// 	//let r_tran = r_trans_factory.create(Box::pin(r_chan));
//    let r_tran = r_trans_factory.create(Box::new(r_chan));
//    //let i_prot = i_proto_factory.create(r_tran);

//    // output protocol and transport
//    //let w_tran = w_trans_factory.create(Box::pin(w_chan));
//    let w_tran = w_trans_factory.create(Box::new(w_chan));
//    //let o_prot = o_proto_factory.create(w_tran);

//    ProcessorWrapper {
//        processor: processor,
//        //i_prot: Arc::new(Mutex::new(i_proto_factory.create(r_tran))),
//        //o_prot: Arc::new(Mutex::new(o_proto_factory.create(w_tran))),
//        i_prot: i_proto_factory.create(r_tran),
//        o_prot: o_proto_factory.create(w_tran),
//    }
// }

// impl<PRC> futures::Future for ProcessorWrapper<PRC>
// where
//     // PRC: TAsyncProcessor + Send + Sync + 'static
//     PRC: TAsyncProcessor
// {
//     type Output = crate::Result<()>;
//     type Output = ();

//     fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
//         let i_prot : &mut Box<dyn TAsyncInputProtocol + Send> = &mut self.i_prot.lock().unwrap();
//         let o_prot : &mut Box<dyn TAsyncOutputProtocol + Send> = &mut self.o_prot.lock().unwrap();
//         let i_prot = &mut self.i_prot;
//         let o_prot = &mut self.o_prot;
//         let me = self.project();
//         let proc = me.processor;
//         loop {
//             match proc
//                 .process(&mut **me.i_prot, 
//                             &mut **me.o_prot) {
//                 Poll::Ready(Ok(t)) => {},
//                 Poll::Pending => return Poll::Pending,
//                 Poll::Ready(Err(err)) => {
//                     match err {
//                         crate::Error::Transport(ref transport_err)
//                         if transport_err.kind == TransportErrorKind::EndOfFile => {
//                             println!("end of file")
//                         }
//                         other => println!("processor completed with error: {:?}", other)
//                     };
//                 return Poll::Ready(Err());
//                 return Poll::Ready(());
//                 },
//             };
//         };
//     }
// }
