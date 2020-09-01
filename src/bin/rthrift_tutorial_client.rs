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
use thrift::protocol::{TBinaryInputProtocol, TBinaryOutputProtocol};
use thrift::transport::{ReadHalf, TFramedReadTransport, TFramedWriteTransport, 
    TBufferedReadTransport, TBufferedWriteTransport, TIoChannel,
                        TTcpChannel, WriteHalf};

use std::time::{SystemTime, Duration};

use rthrift_tutorial::shared::*;
use std::thread;
use std::thread::JoinHandle;
use std::sync::{Arc, Mutex};
use std::vec::Vec;

fn call_client(addr: Arc<String>, port: &u16, loop_num: u64) -> thrift::Result<()> {
    let mut client = new_client(addr.as_ref(), *port)?;
    
    for _i in 0..loop_num {
        let arg1 = 1;
        let arg2 = 2;
        client.add(arg1, arg2)?;
    }
    Ok(())
}

 fn run_test(addr: Arc<String>, port: u16, loop_num: u64, thread_num: u64) -> thrift::Result<()> {
    let mut _time_vec : Vec<u128> = Vec::new();
    let mut _time_vec = Arc::new(Mutex::new(_time_vec));
    
    let mut handler_vec: Vec<JoinHandle<()>> = Vec::new();
    for _i in 0..thread_num {
        let addr = addr.clone();
        let mut _time_vec = _time_vec.clone();
        _time_vec.lock().unwrap().push(0);

        let handler = thread::spawn(move || {
            // thread code
            let time_begin = SystemTime::now();
            
            match call_client(addr, &port, loop_num) {
                Ok(_) => {},
                Err(_) => {println!("[gbd] Call client error here");},
            };

            let time_end = SystemTime::now();
            let mut duration_time = Duration::from_secs(1); 
            let duration = time_end.duration_since(time_begin);
            match duration {
                Ok(_d) => {duration_time = _d;},
                _ => {},
            }
            let time : u128 = duration_time.as_millis();
            _time_vec.lock().unwrap()[_i as usize] = time;
            //println!("elapse: {}", time);
        });
        //print!("after spawn thread: {}", _i);
        handler_vec.push(handler);
    }

    for handler in handler_vec {
        handler.join().unwrap();
    }

    let mut total : u128 = 0;
    for time in _time_vec.lock().unwrap().iter() {
        total += time;
    }

    let qps : f64 = ((thread_num * loop_num) as f64) / (total as f64 / 1000 as f64);

    println!("{} Req/ms", qps);
    Ok(())
}

fn main() {
    let options = clap_app!(rust_tutorial_client =>
        (version: "0.1.0")
        (author: "Apache Thrift Developers <dev@thrift.apache.org>")
        (about: "Thrift Rust tutorial client")
        (@arg host: --host +takes_value "host on which the tutorial server listens")
        (@arg port: --port +takes_value "port on which the tutorial server listens")
        (@arg iter: --iter +takes_value "Iteration Numbers")
        (@arg thread: --thread +takes_value "Thread Numbers")
    );
    let matches = options.get_matches();

     // get any passed-in args or the defaults
    let host = matches.value_of("host").unwrap_or("127.0.0.1");
    let port = value_t!(matches, "port", u16).unwrap_or(9090);
    let iter = value_t!(matches, "iter", u64).unwrap_or(50000);
    let thread_num = value_t!(matches, "thread", u64).unwrap_or(12);

    println!("Client configuration: IP: {}:{}, iter: {} thread_num: {}",
                host, port, iter, thread_num);
    
    let host_arc = Arc::new(host.to_string());

    match run_test(host_arc, port, iter, thread_num) {
        Ok(_ok) => {},
        Err(_err) => {},
    };

    //let s = "Hello World!".to_string();


    /* New a Client */

    /* Send Request */
    
        
    println!("Get the end of test");

}

// type ClientInputProtocol = TBinaryInputProtocol<TFramedReadTransport<ReadHalf<TTcpChannel>>>;
// type ClientOutputProtocol = TBinaryOutputProtocol<TFramedWriteTransport<WriteHalf<TTcpChannel>>>;

type ClientInputProtocol = TBinaryInputProtocol<TBufferedReadTransport<ReadHalf<TTcpChannel>>>;
type ClientOutputProtocol = TBinaryOutputProtocol<TBufferedWriteTransport<WriteHalf<TTcpChannel>>>;


fn new_client
    (
    host: &str,
    port: u16,
) -> thrift::Result<SharedServiceSyncClient<ClientInputProtocol, ClientOutputProtocol>> {
    let mut c = TTcpChannel::new();

    // open the underlying TCP stream
    // println!("connecting to tutorial server on {}:{}", host, port);
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
    Ok(SharedServiceSyncClient::new(i_prot, o_prot))
}
