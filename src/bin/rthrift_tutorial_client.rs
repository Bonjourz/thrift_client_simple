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
use rand::prelude::*;
use std::mem;

use std::path::Path;
use std::fs::File;
use std::io::prelude::*;

fn get_duration(start_time : SystemTime, end_time: SystemTime) -> Duration {
    let duration = end_time.duration_since(start_time);
    let mut duration_time = Duration::from_secs(1);
    match duration {
        Ok(_d) => {duration_time = _d;},
        _ => {},
    }
    duration_time
}

fn get_duration_in_ms(start_time : SystemTime, end_time: SystemTime) -> u64 {
    let res = get_duration(start_time, end_time).as_millis() as u64;
    //println!("[gbd] begin: {:?} end: {:?} duration: {}", start_time, end_time, res);
    res
}

fn get_duration_in_ns(start_time : SystemTime, end_time: SystemTime) -> u64 {
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

fn throughput_test_internal(addr: Arc<String>, port: u16, loop_num : u64, req_per_conn: u64, 
        buf_size : u64) -> thrift::Result<u64> {
    /* First generate packet to send */
    let string_to_send = generate_random_buf(buf_size);
    //let string_to_send : Rc<String> = Rc::new(string_to_send);

    let mut total_time_in_ms = 0;
    for _j in 0..loop_num {
        /* Start client */
        let mut client = new_client(addr.as_ref(), port)?;

        /* Send request */
        let time_begin = SystemTime::now();
        for _i in 0..req_per_conn {
            let string_to_send = string_to_send.clone();
            match client.send_packet(string_to_send) {
                Ok(_res) => {},
                Err(_e) => {},
            }
        }

        let time_end = SystemTime::now();
        total_time_in_ms += get_duration_in_ms(time_begin, time_end);
    }

    /* Return kB/s here */
    Ok(
        ((loop_num * req_per_conn * buf_size / 1024) as f64 / (total_time_in_ms as f64 / 1000 as f64)) as u64
    )
}

fn throughput_test(addr: Arc<String>, port: u16, loop_num: u64, thread_num: u64, req_per_conn: u64, 
    buf_size : u64) -> thrift::Result<()> {
    let mut handler_vec: Vec<JoinHandle<()>> = Vec::new();
    let thp_array : Arc<Mutex<Vec<u64>>> = Arc::new(Mutex::new(Vec::new()));

    /* Spawn threads */
    for _i in 0..thread_num {
        let addr = addr.clone();
        let thp_array = thp_array.clone();
        let handler = thread::spawn( move || {
            match throughput_test_internal(addr, port, loop_num, req_per_conn, buf_size) {
                Ok(thp) => { thp_array.lock().unwrap().push(thp); },
                Err(_e) => { println!("[gbd] run_latency_test error here 1!"); },
            };
        });
        handler_vec.push(handler);
    }

    /* Join threads */
    for handler in handler_vec {
        handler.join().unwrap();
    }

    /* Sort the array */
    let mut total_thp = 0;
    for thp in thp_array.lock().unwrap().iter() {
        total_thp += thp;
    }

    println!("Throughput: {}kB/s", total_thp / thread_num);
    
    Ok(())
}

fn latency_test_internal(addr: Arc<String>, port: u16, req_per_conn: u64, 
    buf_size : u64) -> thrift::Result<Vec<u64>> {
    /* First generate packet to send */
    let string_to_send = generate_random_buf(buf_size);
    
    /* Generate array to store the time distribution */
    let mut time_array : Vec<u64> = Vec::new();
    for _i in 0..req_per_conn { time_array.push(0); }

    /* Start client and send request */
    let mut client = new_client(addr.as_ref(), port)?;

    for _i in 0..req_per_conn {
        let time_begin = SystemTime::now();
        match client.send_packet(string_to_send.clone()) {
            Ok(_res) => {},
            Err(_e) => {},
        }
        let time_end = SystemTime::now();
        time_array[_i as usize] = get_duration_in_ns(time_begin, time_end);
    }
    
    //println!("[gbd] time array 0: {}", time_array[0]);
    Ok(time_array)
}

fn run_latency_test(addr: Arc<String>, port: u16, thread_num: u64, req_per_conn: u64, 
    buf_size : u64) -> std::io::Result<()> {
    let mut handler_vec: Vec<JoinHandle<()>> = Vec::new();
    let time_array: Arc<Mutex<Vec<u64>>> = Arc::new(Mutex::new(Vec::new()));

    /* Spawn threads */
    for _i in 0..thread_num {
        let addr = addr.clone();
        let time_array = time_array.clone();
        let handler = thread::spawn( move || {
            match latency_test_internal(addr, port, req_per_conn, buf_size) {
                Ok(ref mut _vec_ret) => { time_array.lock().unwrap().append(_vec_ret); },
                Err(_e) => { println!("[gbd] run_latency_test error here 1!"); },
            };
        });
        handler_vec.push(handler);
    }

    /* Join threads */
    for handler in handler_vec {
        handler.join().unwrap();
    }

    /* Sort the array */
    time_array.lock().unwrap().sort();

    /* Open file and output the result */
    let mut total_time = 0;
    let path_str = format!("latency_{}t_{}_{}kB_in_ns", 
            thread_num, req_per_conn, buf_size / 1024);
    
    let mut file = File::create(&path_str)?;
    for time in time_array.lock().unwrap().iter() {
        total_time += time;
        file.write_all(format!("{}\n", time).as_bytes())?;
    }

    /* Write the Average Time */
    file.write_all(format!("average time: {}", 
        total_time as f64 / (req_per_conn * thread_num) as f64).as_bytes())?;
    println!("[gbd] arrive here after open a file");

    Ok(())
}

fn test_qps(addr: Arc<String>, port: &u16, loop_num: u64, req_per_conn: u64) -> thrift::Result<u64> {
    let mut time_in_ms : u64 = 0;

    for _i in 0..loop_num {
        let time_begin = SystemTime::now();

        let mut client = new_client(addr.as_ref(), *port)?;
        // let target_val = 3;

        for _i in 0..req_per_conn {
            let arg1 = 1;
            let arg2 = 2;
            match client.add(arg1, arg2) {
				Ok(_val) => {/*assert_eq!(_val, target_val)*/},
				Err(_e) => { println!("error"); }
			};
        }

        let time_end = SystemTime::now();
        time_in_ms += get_duration_in_ms(time_begin, time_end);
    }

    Ok(time_in_ms)
}

 fn run_qps_test(addr: Arc<String>, port: u16, loop_num: u64, thread_num: u64, 
        req_per_conn: u64) -> thrift::Result<()> {
    
    let total_time : Arc<Mutex<u64>> = Arc::new(Mutex::new(0));

    let mut handler_vec: Vec<JoinHandle<()>> = Vec::new();
    for _i in 0..thread_num {
        let addr = addr.clone();

        let total_time = total_time.clone();
        let handler = thread::spawn(move || {
            // thread code
            let mut time_in_ms = 0;
            match test_qps(addr, &port, loop_num, req_per_conn) {
                Ok(_time_in_ms) => { time_in_ms = _time_in_ms; },
                Err(_) => { println!("[gbd] Call client error here"); },
            };

            *total_time.lock().unwrap() += time_in_ms;
            //println!("elapse: {}", time);
        });
        //print!("after spawn thread: {}", _i);
        handler_vec.push(handler);
    }

    for handler in handler_vec {
        handler.join().unwrap();
    }

    let total_time : u64 = *total_time.lock().unwrap();
    let average_secs : u64 = 
        (total_time as f64 / thread_num as f64) as u64 / 1000;
	
	println!("Average Time: {} seconds", average_secs);
    let qps : f64 = ((loop_num * req_per_conn) as f64) / 
		(average_secs as f64);

    println!("{} Req/s", qps);
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
        (@arg reqnum: --reqnum +takes_value "Request Numbers")
        (@arg bufsize: --bufsize +takes_value "Buffer Size in kB")
    );
    let matches = options.get_matches();

     // get any passed-in args or the defaults
    let host = matches.value_of("host").unwrap_or("127.0.0.1");
    let port = value_t!(matches, "port", u16).unwrap_or(9090);
    let loop_num = value_t!(matches, "iter", u64).unwrap_or(5000);
    let thread_num = value_t!(matches, "thread", u64).unwrap_or(12);
    let req_per_conn = value_t!(matches, "reqnum", u64).unwrap_or(5000);
    let buf_size_in_kb = value_t!(matches, "bufsize", u64).unwrap_or(1);

    println!("Client configuration: IP: {}:{}, iter: {} thread_num: {} req_per_conn {}",
                host, port, loop_num, thread_num, req_per_conn);
    
    let host_arc = Arc::new(host.to_string());

    // match run_qps_test(host_arc, port, loop_num, thread_num, req_per_conn) {
    //     Ok(_ok) => {},
    //     Err(_err) => {},
    // };

    let buf_size = buf_size_in_kb * 1024;
    match run_latency_test(host_arc, port, thread_num, req_per_conn, 
        buf_size) {
        Ok(_ok) => {},
        Err(_err) => { println!("run_latency_test err: {:?}", _err); }
    };
    // match run_throughput_test(host_arc, port, loop_num, thread_num, buf_size) {
    //     Ok(_ok) => {},
    //     Err(_err) => {},
    // };
        
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

mod test {

}
