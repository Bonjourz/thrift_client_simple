use serde::{Deserialize, Serialize};
use serde_json;
use std::vec::Vec;
use std::fs::File;
use std::io::prelude::Write;

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
struct User{
    id: i64,
    name: &'static str,
    sex: i32,
    birthday:i32,
    email: &'static str,
    mobile:&'static str,
    address: &'static str,
    icon: &'static str,
    permissions: Vec<i32>,
    status:i32,
    createTime: i64,
    updateTime: i64,
    visHistory: Vec<UserPage>
}

impl User{
    fn new(id:i64)->Self{
        User{
            id:id,
            name:"",
            sex:0,
            birthday:0,
            email:"",
            mobile:"",
            address:"",
            icon:"",
            permissions:vec![4,2,2,1],
            status:0,
            createTime:0,
            updateTime:0,
            visHistory:vec![UserPage::new(1),UserPage::new(2)]
        }
    }
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
struct UserPage{
    pageNo: i32,
    total: i32,
}

#[allow(non_snake_case)]
impl UserPage{
    fn new(pageNo:i32)->Self{
        UserPage{
            pageNo:pageNo,
            total:1,
        }
    }
}

#[derive(Serialize,Deserialize)]
struct Date{
    year:i32,
    month:i8,
    day:i8
}

impl Date{
    fn new()->Self{
        Date{
            year:0,
            month:0,
            day:0
        }
    }
}

#[derive(Serialize,Deserialize)]
struct Address{
    city:&'static str,
    zipcode:i32,
    street:&'static str,
    number:i16
}

impl Address{
    fn new()->Self{
        Address{
            city:"ct",
            zipcode:0,
            street:"st",
            number:0
        }
    }
}

#[derive(Serialize,Deserialize)]
struct School{
    name:&'static str,
    address:Address,
    foundation:Date,
    email_addresses:Vec<&'static str>
}

impl School{
    fn new(name:&'static str)->Self{
        School{
            name:name,
            address:Address::new(),
            foundation:Date::new(),
            email_addresses:vec!["em1","em2","em3"]
        }
    }
}

#[derive(Serialize,Deserialize)]
struct Subject{
    id:i32,
    title:&'static str,
    code:&'static str
}

impl Subject{
    fn new(id:i32)->Self{
        Subject{
            id:id,
            title:"",
            code:""
        }
    }
}

#[derive(Serialize,Deserialize)]
struct Student{
    name:&'static str,
    friends:i32,
    home_address:Address,
    birth_place:Address,
    birth:Date,
    favorite_subjects:Vec<Subject>,
    email_addresses:Vec<&'static str>,
    schools:Vec<School>
}



impl Student{
    fn new(fav_subjects:Vec<Subject>, email_addresses:Vec<&'static str>,schools:Vec<School>)->Self{
        Student{
            name:"",
            friends:1,
            home_address:Address::new(),
            birth_place:Address::new(),
            birth:Date::new(),
            favorite_subjects:fav_subjects,
            email_addresses:email_addresses,
            schools:schools
        }
    }
}

fn main() {
    let mut bm_data_users = Vec::new();
    for i in 1..=100{
        bm_data_users.push(User::new(i));
    }
    let mut bm_students = Vec::new();
    for i in 1..=100{
        let fav_subjects = vec![Subject::new(i)];
        let email_addresses = vec!["em addr"];
        let schools = vec![School::new("school")];
        bm_students.push(Student::new(fav_subjects, email_addresses, schools));
    }

    let bm_data = serde_json::to_vec_pretty(&bm_data_users).unwrap();
    let bm_stus = serde_json::to_vec_pretty(&bm_students).unwrap();
    let bm_data_u8:&[u8] = &bm_data;
    let bm_stus_u8:&[u8] = &bm_stus;
    let mut bm_user_data = File::create("bm_user_data.json").unwrap();
    let _ = bm_user_data.write_all(bm_data_u8);
    let _ = bm_user_data.flush();
    let mut bm_stus_data = File::create("bm_stus_data.json").unwrap();
    let _ = bm_stus_data.write_all(bm_stus_u8);
    let _ = bm_stus_data.flush();
}

