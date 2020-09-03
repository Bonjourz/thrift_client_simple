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

#[allow(unused_imports)]
use thrift::protocol::{TAsyncBinaryInputProtocol, TAsyncBinaryOutputProtocol};
use thrift::server::TAsyncServer;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tokio::net::ToSocketAddrs;

use async_trait::async_trait;
// use thrift::server::TMultiplexedProcessor;

use thrift::transport::{TBufferedReadTransportFactory, TBufferedWriteTransportFactory};
// use thrift::transport::{TFramedReadTransportFactory, TFramedWriteTransportFactory};
use rthrift_tutorial::shared::*;

// fn main() {
//     match run() {
//         Ok(()) => println!("tutorial server ran successfully"),
//         Err(e) => {
//             println!("tutorial server failed with error {:?}", e);
//             std::process::exit(1);
//         }
//     }
// }

#[tokio::main]
pub async fn main() -> thrift::Result<()> {
    let options = clap_app!(rust_tutorial_server =>
        (version: "0.1.0")
        (author: "Apache Thrift Developers <dev@thrift.apache.org>")
        (about: "Thrift Rust tutorial server")
        (@arg host: --host +takes_value "The IP address this server binds")
        (@arg port: --port +takes_value "port on which the tutorial server listens")
        (@arg worker: --worker +takes_value "The Worker Num Server Has")
        
    );
    let matches = options.get_matches();

    let host = matches.value_of("host").unwrap_or("127.0.0.1");
    let port = value_t!(matches, "port", u16).unwrap_or(11235);
    let listen_address = format!("{}:{}", host, port);
    let thread_num = value_t!(matches, "worker", usize).unwrap_or(10);

    //let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(host)), port);
    println!("Server configuaration: addr {}, worker thread: {}", listen_address, thread_num);

    //let i_tran_fact = TBufferedReadTransportFactory::new();
    // let i_tran_fact = TFramedReadTransportFactory::new();
    //let i_prot_fact = TBinaryInputProtocolFactory::new();

    //let o_tran_fact = TBufferedWriteTransportFactory::new();
    // let o_tran_fact = TFramedWriteTransportFactory::new();
    //let o_prot_fact = TBinaryOutputProtocolFactory::new();

    // demux incoming messages
    let processor = SharedServiceSyncProcessor::new(SharedServiceServer {});

    // create the server and start listening
    let mut server = TAsyncServer::new(
        processor,
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 11235),
        thread_num,
    );

    // let processor_multi = CalculatorSyncProcessor::new(CalculatorServer { ..Default::default() });
    // let mut multiplexed_processor = TMultiplexedProcessor::new();
    // multiplexed_processor.register("ping", Box::new(processor_multi), true);
    println!("server serve here");
    server.serve().await;

    Ok(())
}

/// Handles incoming Calculator service calls.
struct SharedServiceServer {}

// since Calculator extends SharedService we have to implement the
// handler for both traits.
//


// SharedService handler
#[async_trait]
impl SharedServiceSyncHandler for SharedServiceServer {
    async fn handle_add(&self, num1: i32, num2: i32) -> thrift::Result<i32> {
        //println!("handling add: n1:{} n2:{}", num1, num2);
        println!("receive num1: {} num2: {}", num1, num2);
        Ok(num1 + num2)
    }
    async fn handle_send_packet(&self, _str: String) -> thrift::Result<String> {
        let output: std::string::String = String::from(_str);
        //println!("output: {}", output);
        Ok(output.clone())
    }

    async fn handle_send_empty(&self) -> thrift::Result<()> {
        Ok(())
    }
}
