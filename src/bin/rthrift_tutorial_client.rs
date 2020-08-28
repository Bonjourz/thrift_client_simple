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

use thrift::protocol::{TBinaryInputProtocol, TBinaryOutputProtocol};
use thrift::transport::{ReadHalf, TFramedReadTransport, TFramedWriteTransport, 
    TBufferedReadTransport, TBufferedWriteTransport, TIoChannel,
                        TTcpChannel, WriteHalf};

use rthrift_tutorial::shared::TSharedServiceSyncClient;
use rthrift_tutorial::tutorial::{CalculatorSyncClient, Operation, TCalculatorSyncClient, Work};

// fn main() {
//     match run() {
//         Ok(()) => println!("tutorial client ran successfully"),
//         Err(e) => {
//             println!("tutorial client failed with error {:?}", e);
//             std::process::exit(1);
//         }
//     }
// }


type ClientInputProtocol = TBinaryInputProtocol<TBufferedReadTransport<ReadHalf<TTcpChannel>>>;
type ClientOutputProtocol = TBinaryOutputProtocol<TBufferedWriteTransport<WriteHalf<TTcpChannel>>>;

async fn async_run_client_one(host : &str, port: u16) -> thrift::Result<()> {
    let mut client = new_client(host, port)?;

     // alright!
    // let's start making some calls

    // let's start with a ping; the server should respond
    println!("ping!");
    client.ping().await?;

    // simple add
    println!("add");
    let res = client.add(1, 2).await?;
    println!("added 1, 2 and got {}", res);

    let logid = 32;

    // let's do...a multiply!
    let res = client
        .calculate(logid, Work::new(7, 8, Operation::MULTIPLY, None)).await?;
    println!("multiplied 7 and 8 and got {}", res);

    // let's get the log for it
    // let res = client.get_struct(32).await?;
    // println!("got log {:?} for operation {}", res, logid);

    // ok - let's be bad :(
    // do a divide by 0
    // logid doesn't matter; won't be recorded
    let res = client.calculate(77, Work::new(2, 0, Operation::DIVIDE, "we bad".to_owned())).await;

    // we should have gotten an exception back
    match res {
        Ok(v) => panic!("should not have succeeded with result {}", v),
        Err(e) => println!("divide by zero failed with error {:?}", e),
    }

    // let's do a one-way call
    println!("zip");
    client.zip().await?;

    // and then close out with a final ping
    println!("ping!");
    client.ping().await?;

    Ok(())
}

async fn run_all_sync() {
    let host1 = String::from("127.0.0.1");
    let host2 = String::from("127.0.0.1");
    // let host3 = String::from("127.0.0.1");
    // let ret_val1 = ::new(ReBox<int>);
    let mut async_1 = async_run_client_one(&host1, 9090);
    let mut async_2 = async_run_client_one(&host2, 10100);
    // let mut async_2 = async_run_client_one(&host3, 10101);
    // let futures = vec![async_1, async_2];
    let (_first, _second) = tokio::join!(
        async_1,
        async_2);
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
    block_on(run_all_sync());
    //tokio::
}

// type ClientInputProtocol = TBinaryInputProtocol<TFramedReadTransport<ReadHalf<TTcpChannel>>>;
// type ClientOutputProtocol = TBinaryOutputProtocol<TFramedWriteTransport<WriteHalf<TTcpChannel>>>;

fn new_client
    (
    host: &str,
    port: u16,
) -> thrift::Result<CalculatorSyncClient<ClientInputProtocol, ClientOutputProtocol>> {
    let mut c = TTcpChannel::new();

    // open the underlying TCP stream
    println!("connecting to tutorial server on {}:{}", host, port);
    c.open(&format!("{}:{}", host, port))?;

    // clone the TCP channel into two halves, one which
    // we'll use for reading, the other for writing
    let (i_chan, o_chan) = c.split()?;

    // wrap the raw sockets (slow) with a buffered transport of some kind
    let i_tran = TBufferedReadTransport::new(i_chan);
    let o_tran = TBufferedWriteTransport::new(o_chan);

    // let i_tran = TFramedReadTransport::new(i_chan);
    // let o_tran = TFramedWriteTransport::new(o_chan);

    // now create the protocol implementations
    let i_prot = TBinaryInputProtocol::new(i_tran, true);
    let o_prot = TBinaryOutputProtocol::new(o_tran, true);

    // we're done!
    Ok(CalculatorSyncClient::new(i_prot, o_prot))
}
