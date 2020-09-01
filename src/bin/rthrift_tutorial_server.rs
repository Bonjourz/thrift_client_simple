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
use thrift::protocol::{TBinaryInputProtocolFactory, TBinaryOutputProtocolFactory};
use thrift::server::TServer;
// use thrift::server::TMultiplexedProcessor;

use thrift::transport::{TBufferedReadTransportFactory, TBufferedWriteTransportFactory};
// use thrift::transport::{TFramedReadTransportFactory, TFramedWriteTransportFactory};
use rthrift_tutorial::shared::*;

fn main() {
    match run() {
        Ok(()) => println!("tutorial server ran successfully"),
        Err(e) => {
            println!("tutorial server failed with error {:?}", e);
            std::process::exit(1);
        }
    }
}

fn run() -> thrift::Result<()> {
    let options = clap_app!(rust_tutorial_server =>
        (version: "0.1.0")
        (author: "Apache Thrift Developers <dev@thrift.apache.org>")
        (about: "Thrift Rust tutorial server")
        (@arg port: --port +takes_value "port on which the tutorial server listens")
    );
    let matches = options.get_matches();

    let port = value_t!(matches, "port", u16).unwrap_or(9090);
    let listen_address = format!("127.0.0.1:{}", port);

    println!("binding to {}", listen_address);

    let i_tran_fact = TBufferedReadTransportFactory::new();
    // let i_tran_fact = TFramedReadTransportFactory::new();
    let i_prot_fact = TBinaryInputProtocolFactory::new();

    let o_tran_fact = TBufferedWriteTransportFactory::new();
    // let o_tran_fact = TFramedWriteTransportFactory::new();
    let o_prot_fact = TBinaryOutputProtocolFactory::new();

    // demux incoming messages
    let processor = SharedServiceSyncProcessor::new(SharedServiceServer {});

    // create the server and start listening
    let mut server = TServer::new(
        i_tran_fact,
        i_prot_fact,
        o_tran_fact,
        o_prot_fact,
        processor,
        10,
    );

    // let processor_multi = CalculatorSyncProcessor::new(CalculatorServer { ..Default::default() });
    // let mut multiplexed_processor = TMultiplexedProcessor::new();
    // multiplexed_processor.register("ping", Box::new(processor_multi), true);

    server.listen(&listen_address)
}

/// Handles incoming Calculator service calls.
struct SharedServiceServer {}

// since Calculator extends SharedService we have to implement the
// handler for both traits.
//


// SharedService handler
impl SharedServiceSyncHandler for SharedServiceServer {
    fn handle_add(&self, num1: i32, num2: i32) -> thrift::Result<i32> {
        //println!("handling add: n1:{} n2:{}", num1, num2);
        Ok(num1 + num2)
    }
}
