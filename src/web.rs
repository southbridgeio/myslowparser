use super::{read_queries, config, queries};

use types::{Query};
use std::sync::Mutex;
use std::thread::{self, sleep};
use processing::{self, qhash};
use rocket;
use rocket::config::{Config, Environment};
use chrono::Utc;

lazy_static! {
    pub static ref wqq: Mutex<Vec<Query>> = Mutex::new(Vec::new());
}

lazy_static! {
    static ref old_response: Mutex<String> = Mutex::new("Please refresh page later".to_string());
}

#[get("/")]
fn all() -> String {
    let mut response: Vec<String> = Vec::new();
    let web_queries = wqq.lock().unwrap();

    let queries_hash = if let Ok(qh_lock) = qhash.try_lock() {
        qh_lock
    } else {
        println!("[{}] WEB: Previous result provided (unable to lock queries count)", Utc::now().to_string());
        return old_response.lock().unwrap().clone();
    };

    let (count_min, count_max) = {
        let cnf = config.lock().unwrap();
        (cnf.count_min, cnf.count_max)
    };

    for (index, query) in web_queries.iter().enumerate() {
        let count = {
            if let Some(qcount) = queries_hash.get(&query.query) {
                *qcount
            } else {
                1
            }
        };

        if count >= count_min && count <= count_max {
            response.push(query.to_string(index + 1, count));
        }
    }

    let mut res = old_response.lock().unwrap();

    *res = response.join("\n");
    res.clone()
}

pub fn invoke_web() {
    let update_thread = thread::spawn(move || {
        let wdelay = config.lock().unwrap().wpd;

        loop {
            read_queries(true);
            let mut qq = queries.lock().unwrap();
            processing::process(&mut qq, true);
            sleep(wdelay * 10);
        }
    });

    let rocket_config = {
        let cnf = config.lock().unwrap();

        println!("\nWeb server running on {}:{}", cnf.web_addr.clone(), cnf.web_port);

        Config::build(Environment::Production)
            .address(cnf.web_addr.clone())
            .port(cnf.web_port)
            .workers(10)
            .finalize().unwrap()
    };

    rocket::custom(rocket_config, false)
        .mount("/", routes![all])
        .launch();

    update_thread.join().expect_err("Can't succesfully end updating thread");
}
