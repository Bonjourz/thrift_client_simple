#![allow(dead_code)]
use thrift::protocol::{
    TCompactInputProtocol, TCompactOutputProtocol
};
use thrift::transport::{TFramedReadTransport, TFramedWriteTransport, TIoChannel, TTcpChannel};

mod benchmark;
use benchmark::TUserServiceSyncClient;

use std::time::Instant;

mod thrift_test;
use thrift_test::TThriftTestSyncClient;

fn run_client(server_addr: &'static str) {
    // Calculating the time elapse
    let time_pt = Instant::now();

    let mut tcp_channel = TTcpChannel::new();
    let ip_str = server_addr;
    let _ = tcp_channel.open(&ip_str);
    let (input_channel, output_channel) = tcp_channel.split().unwrap();

    let buff_read_trans = TFramedReadTransport::new(input_channel);
    let buff_write_trans = TFramedWriteTransport::new(output_channel);

    let input_protocol = TCompactInputProtocol::new(buff_read_trans);
    let output_protocol = TCompactOutputProtocol::new(buff_write_trans);

    let mut client = benchmark::UserServiceSyncClient::new(input_protocol, output_protocol);

    let res = client.user_exist("em".to_owned());
    match res {
        Ok(val) => println!("user exist check result={:?}", val),
        Err(e) => println!("user exist rpc failed, reason={:?}", e.to_string()),
    }

    let added_user = benchmark::User::default();
    let res = client.create_user(added_user);
    match res {
        Ok(val) => println!("create user response={:?}", val),
        Err(e) => println!("user create failed, reason={:?}", e.to_string()),
    }

    let res = client.get_user(100);
    match res {
        Ok(val) => println!("get user response={:?}", val),
        Err(e) => println!("user get failed, reason={:?}", e.to_string()),
    }

    let mut updated_user = benchmark::User::default();
    updated_user.email = Some("user.updated.com".to_owned());

    let res = client.update_user(100, updated_user);
    match res {
        Ok(val) => println!("user updated response={:?}", val),
        Err(e) => println!("user updated failed, reason={:?}", e.to_string()),
    }

    let res = client.list_vis_history(100);
    match res {
        Ok(val) => println!("user vis history response={:?}", val),
        Err(e) => println!("user vis failed, reason={:?}", e.to_string()),
    }

    let res = client.get_blob();
    match res {
        Ok(val) => println!("get blob response={:?}", val),
        Err(e) => println!("get blob failed, reason={:?}", e.to_string()),
    }

    let res = client.get_structs();
    match res {
        Ok(val) => println!("get student structs, response={:?}", val),
        Err(e) => println!("get student failed, reason={:?}", e.to_string()),
    }

    println!("Time elapsed {:?} milliseconds",time_pt.elapsed().as_millis());
}

fn run_client_std_test(server_addr: &'static str){

    let mut tcp_channel = TTcpChannel::new();
    let ip_str = server_addr;
    let _ = tcp_channel.open(&ip_str);
    let (input_channel, output_channel) = tcp_channel.split().unwrap();

    let buff_read_trans = TFramedReadTransport::new(input_channel);
    let buff_write_trans = TFramedWriteTransport::new(output_channel);

    let input_protocol = TCompactInputProtocol::new(buff_read_trans);
    let output_protocol = TCompactOutputProtocol::new(buff_write_trans);

    let mut client = thrift_test::ThriftTestSyncClient::new(input_protocol, output_protocol);

    let time_pt = Instant::now(); // start timer
    for _ in 1..=1000000{
        let _ = client.test_byte(0x00);
    }
    println!("Processing 1000000 RPC, using time {:?} milliseconds",time_pt.elapsed().as_millis());
}

fn main() {
    run_client_std_test("127.0.0.1:60000");
}
