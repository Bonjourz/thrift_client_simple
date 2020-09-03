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

use std::sync::{Arc, Mutex};

use tokio::task::JoinHandle;
use std::collections::VecDeque;
use ringbuf::{RingBuffer, Consumer, Producer};

use std::rc::Rc;
use std::cell::RefCell;
use tokio::net::{TcpStream};

use tokio::sync::mpsc;

use std::time::{SystemTime, Duration};

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

fn get_duration(start_time : SystemTime, end_time: SystemTime) -> Duration {
    let duration = end_time.duration_since(start_time);
    let mut duration_time = Duration::from_secs(0);
    match duration {
        Ok(_d) => {duration_time = _d;},
        _ => { println!("[gbd] get duration error!"); },
    }
    duration_time
}

fn get_duration_in_ms(start_time : SystemTime, end_time: SystemTime) -> u64 {
    let res = get_duration(start_time, end_time).as_millis() as u64;
    res
}

fn get_duration_in_ns(start_time : SystemTime, end_time: SystemTime) -> u64 {
    let res = get_duration(start_time, end_time).as_nanos() as u64;
    res
}


type ClientInputProtocol = TAsyncBinaryInputProtocol<TAsyncBufferedReadTransport<OwnedReadHalf>>;
type ClientOutputProtocol = TAsyncBinaryOutputProtocol<TAsyncBufferedWriteTransport<OwnedWriteHalf>>;

const SERVER_TARGET : &'static str = "127.0.0.1:11235";

async fn async_run_client_one(host : String, port: u16,
    args1: i32) -> thrift::Result<i32> {
    let mut client = new_client().await?;

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

async fn test_qps(ring_prod_arc: Arc<Mutex<Producer<SharedServiceSyncClient<ClientInputProtocol, ClientOutputProtocol>>>>,
    ring_cons_arc: Arc<Mutex<Consumer<SharedServiceSyncClient<ClientInputProtocol, ClientOutputProtocol>>>>,
    total_num: Arc<Mutex<u64>>, target_num: &u64, t_idx: u64) -> thrift::Result<()> {
    let mut my_client;
    let print_num = target_num / 10;
    loop {
        loop {
            if *total_num.lock().unwrap() >= *target_num {
                return Ok(());
            }

            match ring_cons_arc.lock().unwrap().pop() {
                Some(_client) => {
                    my_client = _client;
                    //println!("get ther client");
                    break;
                },
                None => {}, 
            }; 
        }

        my_client.send_empty().await?;
       // println!("client arrive here 1, thread_idx: {}", t_idx);
        ring_prod_arc.lock().unwrap().push(my_client);
        //println!("client arrive here 2");

        let mut lock_arg = total_num.lock().unwrap();
        if *lock_arg >= *target_num {
            return Ok(());
        } else {
            if *lock_arg % print_num == 0 {
                println!("arrive here: {}", *lock_arg);
            }
            *lock_arg += 1;
        }
    }

    Ok(())
}

async fn run_all_async_qps(thread_num: u64, target_num: u64, conn_num: u64) -> thrift::Result<()> {
    //let (mut sender, mut receiver) = mpsc::channel<SharedServiceSyncClient<ClientInputProtocol, ClientOutputProtocol>>(conn_num as usize);
    let rb = RingBuffer::<SharedServiceSyncClient<ClientInputProtocol, ClientOutputProtocol>>::new(conn_num as usize);
    let (mut prod, cons) = rb.split();

    let mut handle_vec: Vec<JoinHandle<()>> = Vec::new();

    for _i in 0..conn_num {
        let client = new_client().await?;
        prod.push(client);
    }

    let initial_total = Arc::new(Mutex::new(0));
    let ring_prod_arc = Arc::new(Mutex::new(prod));
    let ring_cons_arc = Arc::new(Mutex::new(cons));
    let time_begin = SystemTime::now();
    println!("thread num: {}", thread_num);
    for _i in 0..thread_num {
        let initial_total = initial_total.clone();
        let ring_prod_arc = ring_prod_arc.clone();
        let ring_cons_arc = ring_cons_arc.clone();

        let handle = tokio::spawn(async move {
            test_qps(ring_prod_arc, ring_cons_arc, initial_total, &target_num, _i).await;
        });


        handle_vec.push(handle);
    }

    for handle in handle_vec {
        handle.await;
    }

    let time_end = SystemTime::now();
    let time_in_ms = get_duration_in_ms(time_begin, time_end);
    println!("[gbd] the total: {}", *initial_total.lock().unwrap());
    println!("{} Req/s", (target_num) as f64 / (time_in_ms as f64 / 1000 as f64));

    Ok(())
}

fn main() {
    let options = clap_app!(rust_tutorial_client =>
        (version: "0.1.0")
        (author: "Apache Thrift Developers <dev@thrift.apache.org>")
        (about: "Thrift Rust tutorial client")
        (@arg host: --host +takes_value "host on which the tutorial server listens")
        (@arg port: --port +takes_value "port on which the tutorial server listens")
        (@arg thread: --thread +takes_value "Thread Numbers")
        (@arg conn_num: --conn +takes_value "Connection Numbers")
        (@arg reqnum: --reqnum +takes_value "Request Numbers")
        (@arg bufsize: --bufsize +takes_value "Buffer Size in kB")
        (@arg option: --option +takes_value "Option")
    );
    let matches = options.get_matches();

    let loop_num = value_t!(matches, "iter", u64).unwrap_or(5000);
    let thread_num = value_t!(matches, "thread", u64).unwrap_or(12);
    let req_num = value_t!(matches, "reqnum", u64).unwrap_or(50000);
    let buf_size_in_kb = value_t!(matches, "bufsize", u64).unwrap_or(1);
    let option = value_t!(matches, "option", u64).unwrap_or(0);
    let conn_num = value_t!(matches, "conn", u64).unwrap_or(10);

    println!("Client configuration: IP: {}, thread_num: {} req_num {} buf_size per req {} kB option: {}",
                SERVER_TARGET, thread_num, req_num, buf_size_in_kb,
                option);

    let buf_size = buf_size_in_kb * 1024;

    let mut rt = Runtime::new().unwrap();
    match option {
        /* Run qps test */
        0 => {
            match rt.block_on(run_all_async_qps(thread_num, req_num, conn_num)) {
                Ok(_ok) => {},
                Err(_err) => { println!("run_qps_test err: {:?}", _err); },
            };
        },

        /* Run throughput test */
        /*
        1 => {
            match run_throughput_test(thread_num, req_num, conn_num, buf_size) {
                Ok(_ok) => {},
                Err(_err) => { println!("run_throughput_test err: {:?}", _err); },
            };
        }, */

        /* Run latency test */
        /*2 => {
            match run_latency_test(thread_num, req_num, conn_num, buf_size) {
                Ok(_ok) => {},
                Err(_err) => { println!("run_latency_test err: {:?}", _err); }
            };
        }, */

        _op => { println!("Error: Invalide option: {}", _op); },
    }
    
    // get any passed-in args or the defaults
    // tokio::runtim::Runtime

    //let s = "Hello World!".to_string();

    // rt.block_on(run_all_sync());
    // match rt.block_on(run_all_sync()) {
    //     (_x, _y) => {
    //         match _x {
    //             Some(_val1) =>  println!("get the val: {}", _val1),
    //             _ => println!("None val")
    //         }
    //     }
    // }
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

async fn new_client() -> 
    thrift::Result<SharedServiceSyncClient<ClientInputProtocol, ClientOutputProtocol>> {
    let stream = TcpStream::connect(SERVER_TARGET).await?;

    //println!("connecting to tutorial server on {}:{}", host, port);
    //c.open(&format!("{}:{}", host, port))?;

    // clone the TCP channel into two halves, one which
    // we'll use for reading, the other for writing
    let (i_chan, o_chan) = stream.into_split();

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
