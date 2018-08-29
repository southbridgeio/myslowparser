# myslowparser
MySQL slow log parser

```MySQL slow log parser 1.1.5
Developed by Alexander Kozharsky <a.kozharsky@southbridge.io>
Copyright (c) Southbridge, LLC https://southbridge.io
Parses MySQL slow log very fast

USAGE:
    myslowparser [FLAGS] [OPTIONS]

FLAGS:
    -a, --abstract     Abstact strings to |STRING|, numbers to |NUMBER|
    -d, --dedup        Remove query duplicates. Shows only last query
    -h, --help         Prints help information
    -p, --print_cfg    Print current configuration
    -V, --version      Prints version information

OPTIONS:
        --cnt_max <COUNT_MAX>           Query count maximum value
        --cnt_min <COUNT_MIN>           Query count minimum value
        --database <DATABASE>           Database name
    -f, --file <FILE>                   Path to file to parse
    -l, --limit <LIMIT>                 Limit to <LIMIT> first queries
        --lt_max <LOCK_TIME_MAX>        Lock time maximum value
        --lt_min <LOCK_TIME_MIN>        Lock time minimum value
        --qt_max <QUERY_TIME_MAX>       Query time maximum value
        --qt_min <QUERY_TIME_MIN>       Query time minimum value
    -r, --query_regex <REGEX_STRING>    Query regex filter
        --ra_max <ROWS_AFFECTED_MAX>    Rows affected maximum value
        --ra_min <ROWS_AFFECTED_MIN>    Rows affected minimum value
        --re_max <ROWS_EXAMINED_MAX>    Rows examined maximum value
        --re_min <ROWS_EXAMINED_MIN>    Rows examined minimum value
        --rs_max <ROWS_SENT_MAX>        Rows sent maximum value
        --rs_min <ROWS_SENT_MIN>        Rows sent minimum value
    -s, --sort_type <SORT_TYPE>         Sort by column parameter, where SORT_TYPE:
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
                                          cnti - Count inverse
        --ts_max <TIMESTAMP_MAX>        Timestamp range maximum value
                                          format: Unix timestamp or DD/MM/YYYY
        --ts_min <TIMESTAMP_MIN>        Timestamp range minimum value
                                          format: Unix timestamp or DD/MM/YYYY
    -w, --web <ADDR:PORT>               Run web server on <ADDR:PORT>
                                        If ADDR omitted, then listen on 127.0.0.1
                                        Port 0 (zero) to disable feature (disabled by default)
```
