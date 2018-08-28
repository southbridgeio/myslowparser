#![feature(plugin)]
#![plugin(rocket_codegen)]

#[macro_use] extern crate lazy_static;
extern crate regex;
extern crate chrono;
extern crate clap;
extern crate rocket;

mod regs;
mod types;
mod processing;
mod web;

use std::fs::File;
use std::io::{BufReader, Read};
use std::sync::Mutex;
use chrono::TimeZone;
use chrono::prelude::Utc;
use clap::{App, Arg};
use types::{Query, Config, QueriesSortType};
use regex::Regex;
use std::thread::sleep;
use std::time::Duration;

lazy_static! {
    static ref queries: Mutex<Vec<Query>> = Mutex::new(Vec::new());
}

lazy_static! {
    static ref config: Mutex<Config> = Mutex::new(Config::new());
}

fn main() {
    match configure() {
        Err(err) => {
            println!("Can't continue due errors:\n{}", err);
            return;
        }
        _ => {}
    }

    read_queries(false);

    {
        let mut qq = queries.lock().unwrap();
        processing::process(&mut qq, false);
    }

    {
        if config.lock().unwrap().web_port > 0 {
            web::invoke_web();
        }
    }
}

fn read_queries(background: bool) {
    const SLEEP_TIME: Duration = Duration::from_millis(10);

    let log_file = {
        let cnf = config.lock().unwrap();

        cnf.log_file.clone()
    };

    let file = match File::open(&log_file) {
        Ok(file) => file,
        Err(err) => {
            eprintln!("Can't open file {}: {}", log_file, err);
            return;
        }
    };

    let mut buf: [u8; 1] = [0];
    let mut reader = BufReader::new(&file);
    let mut line = String::new();
    let mut new_query = Query::new();
    let mut server_info = String::new();
    let mut server_info_consumed = false;

    while let Ok(_) = reader.read_exact(&mut buf) {
        if buf[0] == 0x0A {
            if !server_info_consumed {
                if !regs::is_info(&line) {
                    server_info += &line;
                    server_info.push_str("\n");
                    line = String::new();
                    continue;
                } else {
                    server_info_consumed = true;
                }
            }

            if regs::is_info(&line) {
                handle_info(&line, &mut new_query);
            } else {
                handle_raw(&line, &mut new_query);
            }

            if new_query.valid() {
                let mut qq = queries.lock().unwrap();
                let query_index = qq.len();
                new_query.finish();

                if regs::is_query_end(&new_query.query) {
                    qq.insert(query_index, new_query);
                }

                if background {
                    sleep(SLEEP_TIME);
                }

                new_query = Query::new();
            }

            line = String::new();
        } else {
            line += &String::from_utf8_lossy(&buf);
        }
    }

    if server_info.len() > 0 && !background {
        let si_split: Vec<&str> = server_info.split("\n").collect();
        let info_string = si_split[0].to_string();
        println!("SERVER INFO: {}\n", info_string.replace(". started with:", ""));
    }
}

fn configure() -> Result<(), String> {
    let mut cnf = config.lock().unwrap();

    let matches = App::new("MySQL slow log parser")
        .version("1.1.5")
        .author("Developed by Alexander Kozharsky <a.kozharsky@southbridge.io>
Copyright (c) Southbridge, LLC https://southbridge.io")
        .about("Parses MySQL slow log very fast")
        .arg(Arg::with_name("file")
            .short("f")
            .long("file")
            .value_name("FILE")
            .help("Path to file to parse"))
        .arg(Arg::with_name("ts_min")
            .long("ts_min")
            .value_name("TIMESTAMP_MIN")
            .help("Timestamp range minimum value
  format: Unix timestamp or DD/MM/YYYY"))
        .arg(Arg::with_name("ts_max")
            .long("ts_max")
            .value_name("TIMESTAMP_MAX")
            .help("Timestamp range maximum value
  format: Unix timestamp or DD/MM/YYYY"))
        .arg(Arg::with_name("database")
            .long("database")
            .value_name("DATABASE")
            .help("Database name"))
        .arg(Arg::with_name("qt_min")
            .long("qt_min")
            .value_name("QUERY_TIME_MIN")
            .help("Query time minimum value"))
        .arg(Arg::with_name("qt_max")
            .long("qt_max")
            .value_name("QUERY_TIME_MAX")
            .help("Query time maximum value"))
        .arg(Arg::with_name("lt_min")
            .long("lt_min")
            .value_name("LOCK_TIME_MIN")
            .help("Lock time minimum value"))
        .arg(Arg::with_name("lt_max")
            .long("lt_max")
            .value_name("LOCK_TIME_MAX")
            .help("Lock time maximum value"))
        .arg(Arg::with_name("rs_min")
            .long("rs_min")
            .value_name("ROWS_SENT_MIN")
            .help("Rows sent minimum value"))
        .arg(Arg::with_name("rs_max")
            .long("rs_max")
            .value_name("ROWS_SENT_MAX")
            .help("Rows sent maximum value"))
        .arg(Arg::with_name("re_min")
            .long("re_min")
            .value_name("ROWS_EXAMINED_MIN")
            .help("Rows examined minimum value"))
        .arg(Arg::with_name("re_max")
            .long("re_max")
            .value_name("ROWS_EXAMINED_MAX")
            .help("Rows examined maximum value"))
        .arg(Arg::with_name("ra_min")
            .long("ra_min")
            .value_name("ROWS_AFFECTED_MIN")
            .help("Rows affected minimum value"))
        .arg(Arg::with_name("ra_max")
            .long("ra_max")
            .value_name("ROWS_AFFECTED_MAX")
            .help("Rows affected maximum value"))
        .arg(Arg::with_name("sort_type")
            .short("s")
            .long("sort_type")
            .value_name("SORT_TYPE")
            .help("Sort by column parameter, where SORT_TYPE:
  ts   - Timestamp
  qt   - Query time
  lt   - Lock time
  rs   - Rows sent
  re   - Rows examined
  ra   - Rows affected
  tsi  - Timestamp inverse
  qti  - Query time inverse
  lti  - Lock time inverse
  rsi  - Rows sent inverse
  rei  - Rows examined inverse
  rai  - Rows affected inverse
  cnt  - Count
  cnti - Count inverse"))
        .arg(Arg::with_name("query_regex")
            .short("r")
            .long("query_regex")
            .value_name("REGEX_STRING")
            .help("Query regex filter"))
        .arg(Arg::with_name("cnt_min")
            .long("cnt_min")
            .value_name("COUNT_MIN")
            .help("Query count minimum value"))
        .arg(Arg::with_name("cnt_max")
            .long("cnt_max")
            .value_name("COUNT_MAX")
            .help("Query count maximum value"))
        .arg(Arg::with_name("limit")
            .short("l")
            .long("limit")
            .value_name("LIMIT")
            .help("Limit to <LIMIT> first queries"))
        .arg(Arg::with_name("abstract")
            .short("a")
            .long("abstract")
            .help("Abstact strings to |STRING|, numbers to |NUMBER|"))
        .arg(Arg::with_name("print_cfg")
            .short("p")
            .long("print_cfg")
            .multiple(true)
            .help("Print current configuration"))
        .arg(Arg::with_name("web")
            .short("w")
            .long("web")
            .value_name("ADDR:PORT")
            .help("Run web server on <ADDR:PORT>
If ADDR omitted, then listen on 127.0.0.1
Port 0 (zero) to disable feature (disabled by default)"))
        .get_matches();

    cnf.log_file = matches.value_of("file").unwrap_or("mysql-slow.log").to_string();

    if let Ok(ts_min) = matches.value_of("ts_min").unwrap_or("-1").parse::<i64>() {
        cnf.timestamp_begin = ts_min;
    } else {
        let ts_min_date = matches.value_of("ts_min").unwrap_or("15/12/1901").to_string();
        let datereg = Regex::new(r"^(?P<dd>\d{2})[/\-.](?P<mm>\d{2})[/\-.](?P<yyyy>\d{4})$").unwrap();

        if let Some(datecapts) = datereg.captures(&ts_min_date) {
            let datestr = format!("{}/{}/{}:00:00:00",
                                  &datecapts["dd"].to_string(),
                                  &datecapts["mm"].to_string(),
                                  &datecapts["yyyy"].to_string());

            let date = Utc.datetime_from_str(&datestr, Query::DT_FORMAT).unwrap();
            cnf.timestamp_begin = date.timestamp();
        } else {
            cnf.add_error("Timestamp range minimum value invalid syntax");
        }
    }

    if let Ok(ts_max) = matches.value_of("ts_max").unwrap_or("-1").parse::<i64>() {
        cnf.timestamp_end = if cnf.timestamp_begin > ts_max || ts_max < 0
            { std::i64::MAX } else { ts_max };
    } else {
        let ts_max_date = matches.value_of("ts_max").unwrap_or("14/12/1901").to_string();
        let datereg = Regex::new(r"^(?P<dd>\d{2})[/\-.](?P<mm>\d{2})[/\-.](?P<yyyy>\d{4})$").unwrap();

        if let Some(datecapts) = datereg.captures(&ts_max_date) {
            let datestr = format!("{}/{}/{}:23:59:59",
                                  &datecapts["dd"].to_string(),
                                  &datecapts["mm"].to_string(),
                                  &datecapts["yyyy"].to_string());

            let date = Utc.datetime_from_str(&datestr, Query::DT_FORMAT).unwrap();
            cnf.timestamp_end = date.timestamp();

            if cnf.timestamp_end < cnf.timestamp_begin {
                cnf.timestamp_end = std::i64::MAX;
            }
        } else {
            cnf.add_error("Timestamp range maximum value invalid syntax");
        }
    }

    cnf.db = matches.value_of("database").unwrap_or("").to_string();

    if let Ok(qt_min) = matches.value_of("qt_min").unwrap_or("-1").parse::<f64>() {
        cnf.query_time_min = qt_min;
    } else {
        cnf.add_error("Query time range minimum value invalid syntax");
    }

    if let Ok(qt_max) = matches.value_of("qt_max").unwrap_or("-1").parse::<f64>() {
        cnf.query_time_max = if cnf.query_time_min > qt_max || qt_max < 0.0
            { std::f64::MAX } else { qt_max };
    } else {
        cnf.add_error("Query time range maximum value invalid syntax");
    }

    if let Ok(lt_min) = matches.value_of("lt_min").unwrap_or("-1").parse::<f64>() {
        cnf.lock_time_min = lt_min;
    } else {
        cnf.add_error("Lock time range minimum value invalid syntax");
    }

    if let Ok(lt_max) = matches.value_of("lt_max").unwrap_or("-1").parse::<f64>() {
        cnf.lock_time_max = if cnf.lock_time_min > lt_max || lt_max < 0.0
            { std::f64::MAX } else { lt_max };
    } else {
        cnf.add_error("Lock time range maximum value invalid syntax");
    }

    if let Ok(rs_min) = matches.value_of("rs_min").unwrap_or("-1").parse::<i64>() {
        cnf.rows_sent_min = rs_min;
    } else {
        cnf.add_error("Rows sent range minimum value invalid syntax");
    }

    if let Ok(rs_max) = matches.value_of("rs_max").unwrap_or("-1").parse::<i64>() {
        cnf.rows_sent_max = if cnf.rows_sent_min > rs_max || rs_max < 0
            { std::i64::MAX } else { rs_max };
    } else {
        cnf.add_error("Rows sent range maximum value invalid syntax");
    }

    if let Ok(re_min) = matches.value_of("re_min").unwrap_or("-1").parse::<i64>() {
        cnf.rows_examined_min = re_min;
    } else {
        cnf.add_error("Rows examined range minimum value invalid syntax");
    }

    if let Ok(re_max) = matches.value_of("re_max").unwrap_or("-1").parse::<i64>() {
        cnf.rows_examined_max = if cnf.rows_examined_min > re_max || re_max < 0
            { std::i64::MAX } else { re_max };
    } else {
        cnf.add_error("Rows examined range maximum value invalid syntax");
    }

    if let Ok(ra_min) = matches.value_of("ra_min").unwrap_or("-1").parse::<i64>() {
        cnf.rows_affected_min = ra_min;
    } else {
        cnf.add_error("Rows affected range minimum value invalid syntax");
    }

    if let Ok(ra_max) = matches.value_of("ra_max").unwrap_or("-1").parse::<i64>() {
        cnf.rows_affected_max = if cnf.rows_affected_min > ra_max || ra_max < 0
            { std::i64::MAX } else { ra_max };
    } else {
        cnf.add_error("Rows affected range maximum value invalid syntax");
    }

    if let Ok(cnt_min) = matches.value_of("cnt_min").unwrap_or("0").parse::<usize>() {
        cnf.count_min = cnt_min;
    } else {
        cnf.add_error("Count range minimum value invalid syntax");
    }

    if let Ok(cnt_max) = matches.value_of("cnt_max").unwrap_or("0").parse::<usize>() {
        cnf.count_max = if cnf.count_min > cnt_max || cnt_max == 0
            { std::usize::MAX } else { cnt_max };
    } else {
        cnf.add_error("Count range maximum value invalid syntax");
    }

    if let Ok(limit) = matches.value_of("limit").unwrap_or("0").parse::<usize>() {
        cnf.limit = if limit > 0 { limit - 1 } else { std::usize::MAX };
    } else {
        cnf.add_error("Limit value invalid syntax");
    }

    if let Some(regex_string) = matches.value_of("query_regex") {
        if let Ok(regex_value) = Regex::new(regex_string) {
            cnf.regex = Some(regex_value);
        } else {
            cnf.add_error("Invalid query regex provided");
        }
    }

    let web = matches.value_of("web").unwrap_or("0").to_string();

    if web != "0" {
        if let Some(addr_port_capts) = regs::addr_port(&web) {
            cnf.web_addr = addr_port_capts["addr"].to_string();
            cnf.web_port = addr_port_capts["port"].parse::<u16>().unwrap();
        } else if let Ok(web_port) = web.parse::<u16>() {
            cnf.web_addr = "127.0.0.1".to_string();
            cnf.web_port = web_port;
        } else {
            cnf.add_error("Invalid web address:port pair provided");
        }
    }

    cnf.abs = matches.occurrences_of("abstract") > 0;

    let sort_type = &*matches.value_of("sort_type").unwrap_or("ts").to_string();

    cnf.sort_type = {
        match sort_type {
            "ts"   => QueriesSortType::Timestamp,
            "qt"   => QueriesSortType::QueryTime,
            "lt"   => QueriesSortType::LockTime,
            "rs"   => QueriesSortType::RowsSent,
            "re"   => QueriesSortType::RowsExamined,
            "ra"   => QueriesSortType::RowsAffected,
            "tsi"  => QueriesSortType::TimestampInverse,
            "qti"  => QueriesSortType::QueryTimeInverse,
            "lti"  => QueriesSortType::LockTimeInverse,
            "rsi"  => QueriesSortType::RowsSentInverse,
            "rei"  => QueriesSortType::RowsExaminedInverse,
            "rai"  => QueriesSortType::RowsAffectedInverse,
            "cnt"  => QueriesSortType::Count,
            "cnti" => QueriesSortType::CountInverse,
            _      => {
                cnf.add_error("Sort type invalid");
                QueriesSortType::Undefined
            }
        }
    };

    let print_matches = matches.occurrences_of("print_cfg");

    if print_matches > 0 {
        println!("{}\n", cnf.to_string());

        if print_matches > 1 {
            cnf.add_error("Interrupted by -pp flag. To disable interrupt, try -p flag");
        }
    }

    if cnf.has_errors() {
        Err(cnf.errors())
    } else {
        Ok(())
    }
}

fn handle_info(line: &String, query: &mut Query) {
    if let Some(time) = regs::date_time(&line) {
        let time_str = format!("{}/{}/{}:{}:{}:{}",
            &time["day"], &time["month"], &time["year"],
            &time["hour"], &time["minute"], &time["second"]);

        if let Ok(time) = Utc.datetime_from_str(&time_str, Query::DT_FORMAT) {
            query.timestamp = time.timestamp();
        }
    }

    if let Some(schema) = regs::schema(&line) {
        query.db = schema["schema"].to_string();
    }

    if let Some(query_time) = regs::query_time(&line) {
        let query_time_str = query_time["query_time"].to_string();

        if let Ok(query_time) = query_time_str.parse::<f64>() {
            query.query_time = query_time;
        }
    }

    if let Some(lock_time) = regs::lock_time(&line) {
        let lock_time_str = lock_time["lock_time"].to_string();

        if let Ok(lock_time) = lock_time_str.parse::<f64>() {
            query.lock_time = lock_time;
        }
    }

    if let Some(rows_sent) = regs::rows_sent(&line) {
        let rows_sent_str = rows_sent["rows_sent"].to_string();

        if let Ok(rows_sent) = rows_sent_str.parse::<i64>() {
            query.rows_sent = rows_sent;
        }
    }

    if let Some(rows_examined) = regs::rows_examined(&line) {
        let rows_examined_str = rows_examined["rows_examined"].to_string();

        if let Ok(rows_examined) = rows_examined_str.parse::<i64>() {
            query.rows_examined = rows_examined;
        }
    }

    if let Some(rows_affected) = regs::rows_affected(&line) {
        let rows_affected_str = rows_affected["rows_affected"].to_string();

        if let Ok(rows_affected) = rows_affected_str.parse::<i64>() {
            query.rows_affected = rows_affected;
        }
    }
}

fn handle_raw(line: &String, query: &mut Query) {
    let mut dirty = false;

    if let Some(db) = regs::db(&line) {
        query.db = db["db"].to_string();
        dirty = true;
    }

    if let Some(timestamp) = regs::timestamp(&line) {
        let timestamp_str = timestamp["timestamp"].to_string();

        if let Ok(timestamp) = timestamp_str.parse::<i64>() {
            query.timestamp = timestamp;
        }

        dirty = true;
    }

    if !dirty {
        if !query.consuming_query {
            query.query = String::new();
            query.consuming_query = true;
        }

        query.query += &*regs::remove_comments(line);
    }

    if query.consuming_query && regs::is_query_end(line) {
        let cnf = config.lock().unwrap();

        if cnf.abs {
            query.query = regs::abs_numbers(&query.query);
            query.query = regs::abs_strings(&query.query);
        }

        query.query = regs::prs_spaces_trim(&query.query);
        query.query_consumed = true;
    }
}
