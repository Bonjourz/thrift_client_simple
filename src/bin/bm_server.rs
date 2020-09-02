#![allow(dead_code)]
use num_cpus;
use std::fs::File;
use std::io::prelude::Read;
use std::vec::Vec;
use thrift::protocol::{TCompactInputProtocolFactory, TCompactOutputProtocolFactory};
use thrift::server::TServer;
use thrift::transport::{TFramedReadTransportFactory, TFramedWriteTransportFactory};

mod benchmark;
use benchmark::{Student, User, UserPage};
use thrift::OrderedFloat;
mod thrift_test;
use thrift_test::{Xtruct,Xtruct2,Numberz,UserId,Insanity};
use std::collections::{BTreeMap,BTreeSet};

#[macro_use]
extern crate lazy_static;
use std::sync::Mutex;

struct MyHandler;

struct StdHandler;

impl thrift_test::ThriftTestSyncHandler for StdHandler{
  fn handle_test_void(&self) -> thrift::Result<()>{
      Ok(())
  }

  fn handle_test_string(&self, thing: String) -> thrift::Result<String>{
      Ok(thing)
  }

  fn handle_test_bool(&self, thing: bool) -> thrift::Result<bool>{
      Ok(thing)
  }

  fn handle_test_byte(&self, thing: i8) -> thrift::Result<i8>{
      Ok(thing)
  }

  fn handle_test_i32(&self, thing: i32) -> thrift::Result<i32>{
      Ok(thing)
  }

  fn handle_test_i64(&self, thing: i64) -> thrift::Result<i64>{
      Ok(thing)
  }

  fn handle_test_double(&self, thing: OrderedFloat<f64>) -> thrift::Result<OrderedFloat<f64>>{
      Ok(thing)
  }

  fn handle_test_binary(&self, thing: Vec<u8>) -> thrift::Result<Vec<u8>>{
      Ok(thing)
  }

  fn handle_test_struct(&self, thing: Xtruct) -> thrift::Result<Xtruct>{
      Ok(thing)
  }

  fn handle_test_nest(&self, thing: Xtruct2) -> thrift::Result<Xtruct2>{
      Ok(thing)
  }

  fn handle_test_map(&self, thing: BTreeMap<i32, i32>) -> thrift::Result<BTreeMap<i32, i32>>{
      Ok(thing)
  }

  fn handle_test_string_map(&self, thing: BTreeMap<String, String>) -> thrift::Result<BTreeMap<String, String>>{
      Ok(thing)
  }

  fn handle_test_set(&self, thing: BTreeSet<i32>) -> thrift::Result<BTreeSet<i32>>{
      Ok(thing)
  }

  fn handle_test_list(&self, thing: Vec<i32>) -> thrift::Result<Vec<i32>>{
      Ok(thing)
  }
  fn handle_test_enum(&self, thing: Numberz) -> thrift::Result<Numberz>{
      Ok(thing)
  }

  fn handle_test_typedef(&self, thing: UserId) -> thrift::Result<UserId>{
      Ok(thing)
  }

  fn handle_test_map_map(&self, hello: i32) -> thrift::Result<BTreeMap<i32, BTreeMap<i32, i32>>>{
      let mut btmap = BTreeMap::new();
      btmap.insert(hello, BTreeMap::default());
      Ok(btmap)
  }

  fn handle_test_insanity(&self, argument: Insanity) -> thrift::Result<BTreeMap<UserId, BTreeMap<Numberz, Insanity>>>{
      let mut btmap = BTreeMap::new();
      let mut ele = BTreeMap::new();
      ele.insert(Numberz::One, argument);
      btmap.insert(0, ele);
      Ok(btmap)
  }
  fn handle_test_multi(&self, _arg0: i8, _arg1: i32, _arg2: i64, _arg3: BTreeMap<i16, String>, _arg4: Numberz, _arg5: UserId) -> thrift::Result<Xtruct>{
      Ok(Xtruct::default())
  }

  fn handle_test_exception(&self, _arg: String) -> thrift::Result<()>{
      Ok(())
  }

  fn handle_test_multi_exception(&self, _arg0: String, _arg1: String) -> thrift::Result<Xtruct>{
      Ok(Xtruct::default())
  }

  fn handle_test_oneway(&self, _seconds_to_sleep: i32) -> thrift::Result<()>{
      Ok(())
  }
}

impl benchmark::UserServiceSyncHandler for MyHandler {
    fn handle_user_exist(&self, email: String) -> thrift::Result<bool> {
        println!("Handle user exist");
        let length: usize = USER_DATA.lock().unwrap().len();
        for i in 0..length {
            println!(
                "checking {} ..., email val={:?}",
                i,
                USER_DATA
                    .lock()
                    .unwrap()
                    .get(i)
                    .unwrap()
                    .clone()
                    .email
                    .unwrap()
            );
            if USER_DATA
                .lock()
                .unwrap()
                .get(i)
                .unwrap()
                .clone()
                .email
                .unwrap()
                == email
            {
                println!("Process finish, response");
                return Ok(true);
            }
        }
        println!("Process finish, response");
        return Ok(false);
    }
    fn handle_create_user(&self, user: User) -> thrift::Result<bool> {
        println!("Handle user create");
        USER_DATA.lock().unwrap().push(Box::new(user));
        println!("Process finish, response");
        Ok(true)
    }
    fn handle_get_user(&self, id: i64) -> thrift::Result<User> {
        println!("Handle user get");
        let length: usize = USER_DATA.lock().unwrap().len();
        for i in 0..length {
            println!("processing user idx={:?}", i);
            if USER_DATA
                .lock()
                .unwrap()
                .get(i)
                .clone()
                .unwrap()
                .id
                .unwrap()
                == id
            {
                println!("Process finish, response");
                return Ok(USER_DATA
                    .lock()
                    .unwrap()
                    .get(i)
                    .unwrap()
                    .clone()
                    .as_ref()
                    .clone());
            }
        }
        println!("Process finish, response");
        return Ok(User::default());
    }
    fn handle_update_user(&self, id: i64, user: User) -> thrift::Result<bool> {
        println!("Handle user update");
        let length: usize = USER_DATA.lock().unwrap().len();
        for i in 0..length {
            if USER_DATA
                .lock()
                .unwrap()
                .get(i)
                .clone()
                .unwrap()
                .id
                .unwrap()
                == id
            {
                *USER_DATA.lock().unwrap().get(i).unwrap().clone() = user.clone();
                break;
            }
        }
        println!("Process finish, response");
        Ok(true)
    }
    fn handle_list_vis_history(&self, id: i64) -> thrift::Result<Vec<UserPage>> {
        println!("Handle vis history");
        let mut vised_page_vec = Vec::new();
        let length: usize = USER_DATA.lock().unwrap().len();
        for i in 0..length {
            if USER_DATA
                .lock()
                .unwrap()
                .get(i)
                .clone()
                .unwrap()
                .id
                .unwrap()
                == id
            {
                if USER_DATA
                    .lock()
                    .unwrap()
                    .get(i)
                    .clone()
                    .unwrap()
                    .vis_history
                    == None
                {
                    break;
                }
                for ele in USER_DATA
                    .lock()
                    .unwrap()
                    .get(i)
                    .clone()
                    .unwrap()
                    .vis_history
                    .clone()
                    .unwrap()
                    .iter()
                {
                    let new_val: UserPage = ele.clone().as_ref().clone();
                    vised_page_vec.push(new_val);
                }
            }
        }
        println!("Process finish, response");
        Ok(vised_page_vec)
    }
    fn handle_get_blob(&self) -> thrift::Result<Vec<u8>> {
        println!("Handle blob get");
        let mut vised_page_vec = Vec::new();
        let length: usize = USER_DATA.lock().unwrap().len();
        for i in 0..length {
            if USER_DATA
                .lock()
                .unwrap()
                .get(i)
                .clone()
                .unwrap()
                .vis_history
                == None
            {
                break;
            }
            for ele in USER_DATA
                .lock()
                .unwrap()
                .get(i)
                .clone()
                .unwrap()
                .vis_history
                .clone()
                .unwrap()
                .iter()
            {
                let new_val: UserPage = ele.clone().as_ref().clone();
                vised_page_vec.push(new_val);
            }
        }
        let binary_userdata = serde_json::to_vec_pretty(&vised_page_vec).unwrap();
        println!("Process finish, response");
        Ok(binary_userdata)
    }
    fn handle_get_structs(&self) -> thrift::Result<Vec<Student>> {
        println!("Handle get students");

        let mut stus = Vec::new();
        let length: usize = USER_DATA.lock().unwrap().len();
        for i in 0..length {
            if STUDENTS_DATA.lock().unwrap().get(i).clone() == None {
                continue;
            }
            let stu = STUDENTS_DATA
                .lock()
                .unwrap()
                .get(i)
                .clone()
                .unwrap()
                .clone();
            stus.push(stu.as_ref().clone());
        }
        println!("Process finish, response");
        Ok(stus)
    }
}

lazy_static! {
    static ref USER_DATA: Mutex<Vec<Box<benchmark::User>>> = {
        let mut file =
            File::open("C:\\Users\\Administrator\\Downloads\\bm\\bm_datagen\\bm_user_data.json")
                .unwrap();
        let mut content = String::new();
        let _ = file.read_to_string(&mut content).unwrap();
        let users: Vec<Box<benchmark::User>> = serde_json::from_str(&content).unwrap();
        println!("user len={}", users.len());
        Mutex::new(users)
    };
    static ref STUDENTS_DATA: Mutex<Vec<Box<benchmark::Student>>> = {
        let mut file =
            File::open("C:\\Users\\Administrator\\Downloads\\bm\\bm_datagen\\bm_stus_data.json")
                .unwrap();
        let mut content = String::new();
        let _ = file.read_to_string(&mut content).unwrap();
        let stus: Vec<Box<benchmark::Student>> = serde_json::from_str(&content).unwrap();
        println!("students data len={:?}", stus.len());
        Mutex::new(stus)
    };
}

fn server_main(bind_addr: &'static str) {
    let listen_addr = bind_addr;
    // let handler = MyHandler;
    let handler = StdHandler;
    let trans_fac_read = TFramedReadTransportFactory::new();
    let trans_fac_write = TFramedWriteTransportFactory::new();
    let trans_comp_input_fac = TCompactInputProtocolFactory::new();
    let trans_comp_output_fac = TCompactOutputProtocolFactory::new();
    // let processor = benchmark::UserServiceSyncProcessor::new(handler);
    let processor = thrift_test::ThriftTestSyncProcessor::new(handler);
    let mut server = TServer::new(
        trans_fac_read,
        trans_comp_input_fac,
        trans_fac_write,
        trans_comp_output_fac,
        processor,
        num_cpus::get(),
    );
    println!("listen addr={}....", listen_addr);
    let _ = server.listen(&listen_addr);
}

fn main() {
    server_main("127.0.0.1:60000");
}
