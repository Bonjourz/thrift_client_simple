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

#[macro_use]
extern crate clap;

extern crate rthrift as thrift;
extern crate rthrift_tutorial;
use futures::executor::block_on;
use std::error::Error;
extern crate tokio;
use tokio::runtime::Runtime;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::io;
use tokio::*;

use std::rc::Rc;
use std::cell::RefCell;
use tokio::net::{TcpStream};

extern crate futures;
// use futures::{Poll, Async, try_ready};

use thrift::protocol::{TAsyncBinaryInputProtocol, TAsyncBinaryOutputProtocol};
use thrift::transport::{TFramedReadTransport, TFramedWriteTransport, 
    TBufferedReadTransport, TBufferedWriteTransport, TIoChannel,
                        TTcpChannel};

use thrift::transport::{
    TAsyncBufferedReadTransport, TAsyncBufferedReadTransportFactory,
    TAsyncBufferedWriteTransport, TAsyncBufferedWriteTransportFactory,
};

use rthrift_tutorial::shared::{TSharedServiceSyncClient, SharedServiceSyncClient};

// fn main() {
//     match run() {
//         Ok(()) => println!("tutorial client ran successfully"),
//         Err(e) => {
//             println!("tutorial client failed with error {:?}", e);
//             std::process::exit(1);
//         }
//     }
// }


type ClientInputProtocol = TAsyncBinaryInputProtocol<TAsyncBufferedReadTransport<OwnedReadHalf>>;
type ClientOutputProtocol = TAsyncBinaryOutputProtocol<TAsyncBufferedWriteTransport<OwnedWriteHalf>>;

async fn async_run_client_one(host : String, port: u16,
    args1: i32) -> thrift::Result<i32> {
    let mut client = new_client(&host, port).await?;

     // alright!
    // let's start making some calls

    // let's start with a ping; the server should respond

    let _gbd_val = 1000;
    match client.add(1, 2).await {
        Ok(_val) => { println!("get the result: {}", _gbd_val); }
        Err(_e) => { println!("error string: {}", _e); }
    };
    println!("val: {}", _gbd_val);
    Ok(1)
}

async fn run_all_sync(/*res_ref1: Rc<RefCell<i32>>, res_ref2: Rc<RefCell<i32>>*/)
    -> (Option<i32>, Option<i32>) {
    
    // let res1 = tokio::spawn(async move {
    //     let host1 = String::from("127.0.0.1");
    //     async_run_client_one(host1, 11235, 2).await
    // });
    let host1 = String::from("127.0.0.1");
    async_run_client_one(host1, 11235, 2).await;

    // let mut res_wrap;
    // let res2 = tokio::spawn(async move {
    //     let host2  = String::from("127.0.0.1");
    //     async_run_client_one(host2, 11235, 3).await
    // });

    // let result_1 = Rc::clone(&res_ref1);
    // let result_2 = Rc::clone(&res_ref2);

    // let res_1 : i32 = res1;
    // let res_2 : i32 = res2;
    // let (_first, _second) = tokio::join!(
    //     res1, res2
    // );
    //let _first = tokio::join!(res1);

    //let test : thrift::Result<i32> = Ok(_first);
    let mut final_ret : i32 = 0;
    // match _first {
    //     // Ok(thrift_result) => final_ret = thrift_result,
    //     Ok(thrift_result) => {
    //         match thrift_result {
    //             Ok(_val) => final_ret = _val,
    //             Err(_e) => println!("error"),
    //         }
    //     },
    //     Err(e) => println!("tutorial client failed with error {:?}", e),
    // }
    println!("[gbd] arrive here");
    (Some(final_ret), Some(2))
}

fn main() {
    let options = clap_app!(rust_tutorial_client =>
        (version: "0.1.0")
        (author: "Apache Thrift Developers <dev@thrift.apache.org>")
        (about: "Thrift Rust tutorial client")
        (@arg host: --host +takes_value "host on which the tutorial server listens")
        (@arg port: --port +takes_value "port on which the tutorial server listens")
    );
    let matches = options.get_matches();

    // get any passed-in args or the defaults
    // tokio::runtim::Runtime
    let mut rt = Runtime::new().unwrap();

    //let s = "Hello World!".to_string();

    // rt.block_on(run_all_sync());
    match rt.block_on(run_all_sync()) {
        (_x, _y) => {
            match _x {
                Some(_val1) =>  println!("get the val: {}", _val1),
                _ => println!("None val")
            }
        }
    }
    println!("arrive here after rt enter");

    /* Barrier here */

    //rt.enter(|| run_all_sync());

    //rt.shutdown_background();

    // block_on(run_all_sync());
    // let handle = tokio::spawn(async move {
    //     run_all_sync
    // }).await.unwrap();

    // 
    // tokio_compat::run_std(run_all_sync());
    //tokio::
}

// type ClientInputProtocol = TBinaryInputProtocol<TFramedReadTransport<ReadHalf<TTcpChannel>>>;
// type ClientOutputProtocol = TBinaryOutputProtocol<TFramedWriteTransport<WriteHalf<TTcpChannel>>>;

async fn new_client
    (
    host: &str,
    port: u16,
) -> thrift::Result<SharedServiceSyncClient<ClientInputProtocol, ClientOutputProtocol>> {
    let mut stream = TcpStream::connect("127.0.0.1:11235").await?;

    //println!("connecting to tutorial server on {}:{}", host, port);
    //c.open(&format!("{}:{}", host, port))?;

    // clone the TCP channel into two halves, one which
    // we'll use for reading, the other for writing
    let (mut i_chan, mut o_chan) = stream.into_split();

    // wrap the raw sockets (slow) with a buffered transport of some kind
    let i_tran = TAsyncBufferedReadTransport::new(i_chan);
    let o_tran = TAsyncBufferedWriteTransport::new(o_chan);

    // let i_tran = TFramedReadTransport::new(i_chan);
    // let o_tran = TFramedWriteTransport::new(o_chan);

    // now create the protocol implementations
    let i_prot = TAsyncBinaryInputProtocol::new(i_tran, false);
    let o_prot = TAsyncBinaryOutputProtocol::new(o_tran, false);

    // we're done!
    //sOk(())
    Ok(SharedServiceSyncClient::new(i_prot, o_prot))
}
