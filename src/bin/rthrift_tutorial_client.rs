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
extern crate tokio;
use tokio::runtime::Runtime;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};

use std::sync::{Arc, Mutex};

use tokio::task::JoinHandle;
use std::collections::VecDeque;
use ringbuf::{RingBuffer, Consumer, Producer};

use tokio::net::{TcpStream};

use rand::prelude::*;
use std::mem;
use std::fs::File;
use std::io::prelude::*;

use std::time::{Instant, Duration};

use thrift::protocol::{TAsyncBinaryInputProtocol, TAsyncBinaryOutputProtocol};
use thrift::transport::{TFramedReadTransport, TFramedWriteTransport, 
    TBufferedReadTransport, TBufferedWriteTransport, TIoChannel,
                        TTcpChannel};

use thrift::transport::{
    TAsyncBufferedReadTransport, TAsyncBufferedReadTransportFactory,
    TAsyncBufferedWriteTransport, TAsyncBufferedWriteTransportFactory,
};

use rthrift_tutorial::shared::{TSharedServiceSyncClient, SharedServiceSyncClient};

use pprof;

fn get_duration(start_time : Instant, end_time: Instant) -> Duration {
    end_time.duration_since(start_time)
}

fn get_duration_in_ms(start_time : Instant, end_time: Instant) -> u64 {
    let res = get_duration(start_time, end_time).as_millis() as u64;
    //println!("[gbd] begin: {:?} end: {:?} duration: {}", start_time, end_time, res);
    res
}

fn get_duration_in_ns(start_time : Instant, end_time: Instant) -> u64 {
    let res = get_duration(start_time, end_time).as_nanos() as u64;
    //println!("[gbd] begin: {:?} end: {:?} duration: {}", start_time, end_time, res);
    res
}

fn generate_random_buf(buf_size : u64) -> String {
    let mut string_to_send = String::from("");
    let mut rng = rand::thread_rng();
    let mut rand_num : u64;
    for _i in 0..(buf_size / (mem::size_of::<u64>() as u64)) {
        rand_num = rng.gen(); // Random Number
        string_to_send.push_str(&rand_num.to_string());
    }

    string_to_send
}


type ClientInputProtocol = TAsyncBinaryInputProtocol<TAsyncBufferedReadTransport<OwnedReadHalf>>;
type ClientOutputProtocol = TAsyncBinaryOutputProtocol<TAsyncBufferedWriteTransport<OwnedWriteHalf>>;

const SERVER_TARGET : &'static str = "127.0.0.1:11235";

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
        // let result = my_client.add(1, 2).await?;
        // assert_eq!(result, 3);
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
    let time_begin = Instant::now();
    println!("thread num: {}", thread_num);
    for _i in 0..thread_num {
        let initial_total = initial_total.clone();
        let ring_prod_arc = ring_prod_arc.clone();
        let ring_cons_arc = ring_cons_arc.clone();

        let handle = tokio::spawn(async move {
            match test_qps(ring_prod_arc, ring_cons_arc, initial_total, &target_num, _i).await {
                Ok(_) => {},
                Err(_) => { println!("[gbd] error"); },
            };
        });


        handle_vec.push(handle);
    }

    for handle in handle_vec {
        handle.await;
    }

    let time_end = Instant::now();
    let time_in_ms = get_duration_in_ms(time_begin, time_end);
    println!("[gbd] the total: {}", *initial_total.lock().unwrap());
    println!("{} Req/s", (target_num) as f64 / (time_in_ms as f64 / 1000 as f64));

    Ok(())
}

async fn latency_test_internal(vec_queue: Arc<Mutex<VecDeque<SharedServiceSyncClient<ClientInputProtocol, ClientOutputProtocol>>>>,
    total_num: Arc<Mutex<u64>>, target_num: &u64, buf_size: u64) -> thrift::Result<Vec<u64>> {
        /* First generate packet to send */
    let string_to_send = generate_random_buf(buf_size);

    /* Generate array to store the time distribution */
    let mut time_array : Vec<u64> = Vec::new();

    let mut my_client;
    let print_num = target_num / 10;
    loop {
        loop {
            if *total_num.lock().unwrap() >= *target_num {
                return Ok(time_array);
            }

            match vec_queue.lock().unwrap().pop_front() {
                Some(_client) => {
                    my_client = _client;
                    break;
                },
                None => {}, 
            }; 
            //println!("capacity: {}", vec_queue.lock().unwrap().capacity());
        }
        let time_begin = Instant::now();
        my_client.send_packet(string_to_send.clone()).await?;
        let time_end = Instant::now();

        time_array.push(get_duration_in_ns(time_begin, time_end));
        vec_queue.lock().unwrap().push_back(my_client);

        let mut lock_arg = total_num.lock().unwrap();
        if *lock_arg >= *target_num {
            return Ok(time_array);
        } else {
            if *lock_arg % print_num == 0 {
                println!("arrie here: {}", *lock_arg);
            }
            *lock_arg += 1;
        }
    }
    
    //println!("[gbd] time array 0: {}", time_array[0]);
    Ok(time_array)
}

async fn run_all_async_latency(thread_num: u64, req_num: u64, conn_num: u64,
    buf_size : u64) -> thrift::Result<()> {
    let connect_vec = Arc::new(Mutex::new(VecDeque::new()));
    /* Create connections */
    for _i in 0..conn_num {
        let client = new_client().await?;
        connect_vec.lock().unwrap().push_back(client);
    }

    /* Spawn threads */
    let mut handler_vec: Vec<JoinHandle<()>> = Vec::new();
    let time_array: Arc<Mutex<Vec<u64>>> = Arc::new(Mutex::new(Vec::new()));
    let total_num = Arc::new(Mutex::new(0));

    for _i in 0..thread_num {
        let connect_vec = connect_vec.clone();
        let time_array = time_array.clone();
        let total_num = total_num.clone();
        let handler = tokio::spawn(async move {
            match latency_test_internal(connect_vec, total_num, &req_num, buf_size).await {
                Ok(ref mut _vec_ret) => { time_array.lock().unwrap().append(_vec_ret); },
                Err(_e) => { println!("[gbd] error"); }
            };
        });
        handler_vec.push(handler);
    }

     /* Join threads */
    for handler in handler_vec {
        handler.await;
    }

    /* Sort the array */
    time_array.lock().unwrap().sort();

    /* Open file and output the result */
    let mut total_time = 0;
    let path_str = format!("latency_{}t_{}_{}kB_in_ns", 
            thread_num, req_num, buf_size / 1024);
    
    let mut file : std::fs::File = File::create(&path_str)?;
    for time in time_array.lock().unwrap().iter() {
        total_time += time;
        file.write_all(format!("{}\n", time).as_bytes())?;
    }

    /* Write the Average Time */
    let average_time = (total_time as f64 / req_num as f64) as u64;
    file.write_all(format!("average time: {} in ns", 
                    average_time).as_bytes())?;
    println!("[gbd] arrive here after open a file");



    Ok(())
}

async fn throughput_test_internal(vec_queue: Arc<Mutex<VecDeque<SharedServiceSyncClient<ClientInputProtocol, ClientOutputProtocol>>>>,
    total_num: Arc<Mutex<u64>>, target_num: &u64, buf_size: u64) -> thrift::Result<()> {
    /* First generate packet to send */
    let string_to_send = generate_random_buf(buf_size);

    let mut my_client;
    let print_num = target_num / 10;
    loop {
        loop {
            if *total_num.lock().unwrap() >= *target_num {
                return Ok(());
            }

            match vec_queue.lock().unwrap().pop_front() {
                Some(_client) => {
                    my_client = _client;
                    break;
                },
                None => {}, 
            }; 
            //println!("capacity: {}", vec_queue.lock().unwrap().capacity());
        }
        my_client.send_packet(string_to_send.clone()).await?;
        vec_queue.lock().unwrap().push_back(my_client);

        let mut lock_arg = total_num.lock().unwrap();
        if *lock_arg >= *target_num {
            return Ok(());
        } else {
            if *lock_arg % print_num == 0 {
                println!("arrie here: {}", *lock_arg);
            }
            *lock_arg += 1;
        }
    }

    /* Return kB/s here */
    Ok(())
}

async fn run_all_async_throughput(thread_num: u64, target_num: u64, conn_num: u64, 
    buf_size : u64) -> thrift::Result<()> {
    let mut handler_vec: Vec<JoinHandle<()>> = Vec::new();
    
    let connect_vec = Arc::new(Mutex::new(VecDeque::new()));
        /* Create connections */
    for _i in 0..conn_num {
        let client = new_client().await?;
        connect_vec.lock().unwrap().push_back(client);
    }

    /* Spawn threads */
    let total_num = Arc::new(Mutex::new(0));
    let begin_time = Instant::now();

    for _i in 0..thread_num {
        let total_num = total_num.clone();
        let connect_vec = connect_vec.clone();
        
        let handler = tokio::spawn(async move {
            match throughput_test_internal(connect_vec, total_num, &target_num, buf_size).await {
                Ok(_) => { },
                Err(_e) => { println!("[gbd] run_throughput_test error here 1!"); },
            };
        });
        handler_vec.push(handler);
    }

     /* Join threads */
    for handler in handler_vec {
        handler.await;
    }
    let end_time = Instant::now();

    /* Sort the array */
    let total_time_in_ms = get_duration_in_ms(begin_time, end_time);
    let total_thp = (target_num * buf_size / 1024) as f64 / 
            (total_time_in_ms as f64 / 1000 as f64);

    println!("Throughput: {}kB/s", total_thp);

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
    let thread_num = value_t!(matches, "thread", u64).unwrap_or(1000);
    let req_num = value_t!(matches, "reqnum", u64).unwrap_or(5000000);
    let buf_size_in_kb = value_t!(matches, "bufsize", u64).unwrap_or(1);
    let option = value_t!(matches, "option", u64).unwrap_or(1);
    let conn_num = value_t!(matches, "conn", u64).unwrap_or(1000);

    println!("Client configuration: IP: {}, thread_num: {} req_num {} buf_size per req {} kB option: {}",
                SERVER_TARGET, thread_num, req_num, buf_size_in_kb,
                option);

    let buf_size = buf_size_in_kb * 1024;

    let mut rt = Runtime::new().unwrap();
    //let guard = pprof::ProfilerGuard::new(100).unwrap();

    match option {
        /* Run qps test */
        0 => {
            match rt.block_on(run_all_async_qps(thread_num, req_num, conn_num)) {
                Ok(_ok) => {},
                Err(_err) => { println!("run_qps_test err: {:?}", _err); },
            };
        },

        /* Run throughput test */
        1 => {
            match rt.block_on(run_all_async_throughput(thread_num, req_num, conn_num, buf_size)) {
                Ok(_ok) => {},
                Err(_err) => { println!("run_throughput_test err: {:?}", _err); },
            };
        },

        /* Run latency test */
        2 => {
            match rt.block_on(run_all_async_latency(thread_num, req_num, conn_num, buf_size)) {
                Ok(_ok) => {},
                Err(_err) => { println!("run_latency_test err: {:?}", _err); }
            };
        },

        _op => { println!("Error: Invalide option: {}", _op); },
    }
    
    // get any passed-in args or the defaults
    // tokio::runtim::Runtime

    println!("arrive here after rt enter");

    // match guard.report().build() {
    //     Ok(report) => {
    //         let file = File::create("flamegraph.svg").unwrap();
    //         report.flamegraph(file).unwrap();

    //         //println!("report: {}", &report);
    //     }
    //     Err(_) => {}
    // };

    /* Barrier here */

    //rt.enter(|| run_all_sync());

    //rt.shutdown_background();

    // block_on(run_all_sync());
    // let handle = tokio::spawn(async move {
    //     run_all_sync
    // }).await.unwrap();
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
