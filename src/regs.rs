use regex::{Captures, Regex};

pub fn is_info(line: &String) -> bool {
    lazy_static! {
        static ref regex: Regex = Regex::new(r"^# .*$").unwrap();
    }

    regex.is_match(line)
}

pub fn schema(line: &String) -> Option<Captures> {
    lazy_static! {
        static ref regex: Regex = Regex::new(r"^# .*Schema: (?P<schema>[^\s]+).*$").unwrap();
    }

    regex.captures(line)
}

pub fn db(line: &String) -> Option<Captures> {
    lazy_static! {
        static ref regex: Regex = Regex::new(r"^use (?P<db>.+);$").unwrap();
    }

    regex.captures(line)
}

pub fn date_time(line: &String) -> Option<Captures> {
    lazy_static! {
        static ref regex: Regex = Regex::new(r"^# .*Time: (?P<year>\d{2})(?P<month>\d{2})(?P<day>\d{2})[^\d]+(?P<hour>\d+).(?P<minute>\d{2}).(?P<second>\d{2}).*$").unwrap();
    }

    regex.captures(line)
}

pub fn query_time(line: &String) -> Option<Captures> {
    lazy_static! {
        static ref regex: Regex = Regex::new(r"^# .*Query_time: (?P<query_time>\d*\.\d+).*$").unwrap();
    }

    regex.captures(line)
}

pub fn lock_time(line: &String) -> Option<Captures> {
    lazy_static! {
        static ref regex: Regex = Regex::new(r"# .*Lock_time: (?P<lock_time>\d*\.\d+).*$").unwrap();
    }

    regex.captures(line)
}

pub fn rows_sent(line: &String) -> Option<Captures> {
    lazy_static! {
        static ref regex: Regex = Regex::new(r"^# .*Rows_sent: (?P<rows_sent>\d+).*$").unwrap();
    }

    regex.captures(line)
}

pub fn rows_examined(line: &String) -> Option<Captures> {
    lazy_static! {
        static ref regex: Regex = Regex::new(r"^# .*Rows_examined: (?P<rows_examined>\d+).*$").unwrap();
    }

    regex.captures(line)
}

pub fn rows_affected(line: &String) -> Option<Captures> {
    lazy_static! {
        static ref regex: Regex = Regex::new(r"^# .*Rows_affected: (?P<rows_affected>\d+).*$").unwrap();
    }

    regex.captures(line)
}

pub fn timestamp(line: &String) -> Option<Captures> {
    lazy_static! {
        static ref regex: Regex = Regex::new(r"SET timestamp=(?P<timestamp>\d+);$").unwrap();
    }

    regex.captures(line)
}

pub fn addr_port(line: &String) -> Option<Captures> {
    lazy_static! {
        static ref regex: Regex = Regex::new(r"(?P<addr>\d+\.\d+\.\d+\.\d+):(?P<port>\d+)").unwrap();
    }

    regex.captures(line)
}

pub fn is_query_end(line: &String) -> bool {
    lazy_static! {
        static ref regex: Regex = Regex::new(r"^.*;$").unwrap();
    }

    !regex.find(line).is_none()
}

pub fn abs_numbers(line: &String) -> String {
    lazy_static! {
        static ref regex1: Regex = Regex::new(r"\s*=\s*(?P<num>\d+)\s*").unwrap();
    }

    lazy_static! {
        static ref regex2: Regex = Regex::new(r"\s\d+").unwrap();
    }

    let ret: String = regex1.replace_all(line, " = |NUMBER| ").into();

    regex2.replace_all(&ret, " |NUMBER| ").into()
}

pub fn abs_strings(line: &String) -> String {
    lazy_static! {
        static ref regex: Regex = Regex::new(r"(?P<str>'.*?')").unwrap();
    }

    regex.replace_all(line, " |STRING| ").into()
}

pub fn prs_spaces_trim(line: &String) -> String {
    lazy_static! {
        static ref regex1: Regex = Regex::new(r"\(\s+").unwrap();
    }

    lazy_static! {
        static ref regex2: Regex = Regex::new(r"\s+\)").unwrap();
    }

    let ret: String = regex1.replace_all(line, "(").into();

    regex2.replace_all(&ret, ")").into()
}

pub fn remove_comments(line: &String) -> String {
    lazy_static! {
        static ref aster_regex: Regex = Regex::new(r"/\*[^!].+?\*/").unwrap();
    }

    lazy_static! {
        static ref dash_regex: Regex = Regex::new(r"-- .*").unwrap();
    }

    lazy_static! {
        static ref hash_regex: Regex = Regex::new(r"# .*").unwrap();
    }

    let aster_ret: String = aster_regex.replace_all(line, "").into();
    let dash_ret: String = dash_regex.replace_all(&aster_ret, "").into();

    hash_regex.replace_all(&dash_ret, "").into()
}
