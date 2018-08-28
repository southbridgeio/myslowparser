use super::{read_queries, config, queries};

use types::{Query};
use std::sync::Mutex;
use std::thread;
use processing::{self, qhash};
use rocket;
use rocket::config::{Config, Environment};

lazy_static! {
    pub static ref wqq: Mutex<Vec<Query>> = Mutex::new(Vec::new());
}

#[get("/")]
fn all() -> String {
    let mut response: Vec<String> = Vec::new();
    let web_queries = wqq.lock().unwrap();
    let queries_hash = qhash.lock().unwrap();
    let cnf = config.lock().unwrap();

    for (index, query) in web_queries.iter().enumerate() {
        let count = {
            if let Some(qcount) = queries_hash.get(&query.query) {
                *qcount
            } else {
                0
            }
        };

        if count >= cnf.count_min && count <= cnf.count_max {
            response.push(query.to_string(index + 1, count));
        }
    }

    response.join("\n")
}

pub fn invoke_web() {
    let update_thread = thread::spawn(move || {
        loop {
            read_queries(true);
            let mut qq = queries.lock().unwrap();
            processing::process(&mut qq, true);
        }
    });

    let rocket_config = {
        let cnf = config.lock().unwrap();

        println!("\nWeb server running on {}:{}", cnf.web_addr.clone(), cnf.web_port);

        Config::build(Environment::Production)
            .address(cnf.web_addr.clone())
            .port(cnf.web_port)
            .finalize().unwrap()
    };

    rocket::custom(rocket_config, false)
        .mount("/", routes![all])
        .launch();

    update_thread.join().expect_err("Can't succesfully end updating thread");
}
