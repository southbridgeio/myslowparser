use chrono::TimeZone;
use chrono::prelude::Utc;
use std::sync::Mutex;
use regex::Regex;

struct QueryStore {
    pub db: String,
    pub timestamp: i64
}

impl QueryStore {
    pub fn new() -> Self {
        Self {
            db: "?".to_string(),
            timestamp: -1
        }
    }
}

lazy_static! {
    static ref store: Mutex<QueryStore> = Mutex::new(QueryStore::new());
}

pub enum QueriesSortType {
    Timestamp = 0,
    QueryTime,
    LockTime,
    RowsSent,
    RowsExamined,
    RowsAffected,
    TimestampInverse,
    QueryTimeInverse,
    LockTimeInverse,
    RowsSentInverse,
    RowsExaminedInverse,
    RowsAffectedInverse,
    Count,
    CountInverse,
    Undefined
}

impl ToString for QueriesSortType {
    fn to_string(&self) -> String {
        match self {
            &QueriesSortType::Timestamp => "Timestamp".to_string(),
            &QueriesSortType::QueryTime => "Query time".to_string(),
            &QueriesSortType::LockTime => "Lock time".to_string(),
            &QueriesSortType::RowsSent => "Rows sent".to_string(),
            &QueriesSortType::RowsExamined => "Rows examined".to_string(),
            &QueriesSortType::RowsAffected => "Rows affected".to_string(),
            &QueriesSortType::TimestampInverse => "Timestamp inverse".to_string(),
            &QueriesSortType::QueryTimeInverse => "Query time inverse".to_string(),
            &QueriesSortType::LockTimeInverse => "Lock time inverse".to_string(),
            &QueriesSortType::RowsSentInverse => "Rows sent inverse".to_string(),
            &QueriesSortType::RowsExaminedInverse => "Rows examined inverse".to_string(),
            &QueriesSortType::RowsAffectedInverse => "Rows affected inverse".to_string(),
            &QueriesSortType::Count => "Count".to_string(),
            &QueriesSortType::CountInverse => "Count inverse".to_string(),
            &QueriesSortType::Undefined => "Undefined".to_string()
        }
    }
}

pub struct Config {
    pub log_file: String,
    pub timestamp_begin: i64,
    pub timestamp_end: i64,
    pub db: String,
    pub query_time_min: f64,
    pub query_time_max: f64,
    pub lock_time_min: f64,
    pub lock_time_max: f64,
    pub rows_sent_min: i64,
    pub rows_sent_max: i64,
    pub rows_examined_min: i64,
    pub rows_examined_max: i64,
    pub rows_affected_min: i64,
    pub rows_affected_max: i64,
    pub sort_type: QueriesSortType,
    pub regex: Option<Regex>,
    pub count_min: usize,
    pub count_max: usize,
    pub limit: usize,
    pub abs: bool,
    pub web_addr: String,
    pub web_port: u16,
    errors: Vec<&'static str>
}

impl Config {
    pub fn new() -> Self {
        Self {
            log_file: "mysql-slow.log".to_string(),
            timestamp_begin: -1,
            timestamp_end: -1,
            db: "".to_string(),
            query_time_min: -1.0,
            query_time_max: -1.0,
            lock_time_min: -1.0,
            lock_time_max: -1.0,
            rows_sent_min: -1,
            rows_sent_max: -1,
            rows_examined_min: -1,
            rows_examined_max: -1,
            rows_affected_min: -1,
            rows_affected_max: -1,
            sort_type: QueriesSortType::Timestamp,
            regex: None,
            count_min: 0,
            count_max: 0,
            limit: 0,
            abs: false,
            web_addr: String::new(),
            web_port: 0,
            errors: Vec::new()
        }
    }

    pub fn add_error(&mut self, err: &'static str) {
        let err_index = self.errors.len();
        self.errors.insert(err_index, err);
    }

    pub fn has_errors(&self) -> bool {
        self.errors.len() != 0
    }

    pub fn errors(&self) -> String {
        let mut errors_string = "\t".to_string();

        errors_string.push_str(&self.errors.join("\n\t"));
        errors_string.push_str("\n\nYou can use -p flag to print configuration");
        errors_string
    }
}

impl ToString for Config {
    fn to_string(&self) -> String {
        format!("CONFIGURATION:
\tLog file: \"{}\"
\tDatabase: \"{}\"
\tTimestamp range: {} - {}
\tQuery time range: {} - {}
\tLock time range: {} - {}
\tRows sent range: {} - {}
\tRows examined range: {} - {}
\tRows affected range: {} - {}
\tSort type: {}
\tQuery regex: {:?}
\tCount range: {} - {}
\tLimit: first {}
\tStr & num abstract: {}
\tWeb address: \"{}\"
\tWeb port: {}",
        self.log_file,
        self.db,
        self.timestamp_begin, self.timestamp_end,
        self.query_time_min, self.query_time_max,
        self.lock_time_min, self.lock_time_max,
        self.rows_sent_min, self.rows_sent_max,
        self.rows_examined_min, self.rows_examined_max,
        self.rows_affected_min, self.rows_affected_max,
        self.sort_type.to_string(),
        self.regex,
        self.count_min, self.count_max,
        if self.limit < super::std::usize::MAX { self.limit + 1 } else { self.limit },
        self.abs,
        self.web_addr,
        self.web_port)
    }
}

pub struct Query {
    pub timestamp: i64,
    pub db: String,
    pub query_time: f64,
    pub lock_time: f64,
    pub rows_sent: i64,
    pub rows_examined: i64,
    pub rows_affected: i64,
    pub query: String,
    pub consuming_query: bool,
    pub query_consumed: bool
}

impl Query {
    pub const DT_FORMAT: &'static str = "%d/%m/%Y:%H:%M:%S";

    pub fn new() -> Self {
        Self {
            timestamp: -1,
            db: "?".to_string(),
            query_time: -1.0,
            lock_time: -1.0,
            rows_sent: -1,
            rows_examined: -1,
            rows_affected: -1,
            query: "?".to_string(),
            consuming_query: false,
            query_consumed: false
        }
    }

    pub fn valid(&self) -> bool {
        self.query_consumed && self.query != "?"
    }

    pub fn finish(&mut self) {
        let mut st = store.lock().unwrap();

        if self.db == "?" {
            self.db = st.db.clone();
        } else {
            st.db = self.db.clone();
        }

        if self.timestamp < 0 {
            self.timestamp = st.timestamp;
        } else {
            st.timestamp = self.timestamp;
        }

        self.query = self.query
            .replace("\r", " ")
            .replace("\n", " ")
            .replace("\t", " ")
            .trim().to_string();

        while self.query.contains("  ") {
            self.query = self.query.replace("  ", " ");
        }

        while self.query.contains(",|STRING|") {
            self.query = self.query.replace(",|STRING|", ", |STRING|");
        }
    }

    pub fn to_string(&self, index: usize, count: usize) -> String {
        let mut buf = format!("> #{} | DATE_TIME: ", index.to_string());

        if self.timestamp >= 0 {
            let date_time = Utc.timestamp(self.timestamp, 0);
            buf.push_str(&date_time.format(Self::DT_FORMAT).to_string());
        } else {
            buf.push_str("?");
        }

        buf.push_str(&format!(" | DATABASE: {}\n>> QUERY_TIME: ", &self.db));

        if self.query_time >= 0.0 {
            buf.push_str(&self.query_time.to_string());
        } else {
            buf.push_str("?");
        }

        buf.push_str(" | ROWS_EXAMINED: ");

        if self.rows_examined >= 0 {
            buf.push_str(&self.rows_examined.to_string());
        } else {
            buf.push_str("?");
        }

        buf.push_str(" | ROWS_AFFECTED: ");

        if self.rows_affected >= 0 {
            buf.push_str(&self.rows_affected.to_string());
        } else {
            buf.push_str("?");
        }

        buf.push_str("\n>>> ROWS_SENT: ");

        if self.rows_sent >= 0 {
            buf.push_str(&self.rows_sent.to_string());
        } else {
            buf.push_str("?");
        }

        buf.push_str(" | LOCK_TIME: ");

        if self.lock_time >= 0.0 {
            buf.push_str(&self.lock_time.to_string());
        } else {
            buf.push_str("?");
        }

        buf.push_str(&format!(" | COUNT: {}", count.to_string()));
        buf.push_str(&format!("\n{}\n", self.query));

        buf
    }
}
