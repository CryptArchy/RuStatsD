use std::io;
use std::result;
use std::str::{from_utf8, FromStr};
use std::time::Duration;
use std::fmt;
use std::num;
use std::vec::Vec;

#[derive(Debug, Clone, PartialEq)]
pub struct ParseMessageError {
    _priv: (),
}

impl fmt::Display for ParseMessageError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        "provided string was not a properly formated statsd message".fmt(f)
    }
}

impl From<num::ParseIntError> for ParseMessageError {
    fn from(err: num::ParseIntError) -> ParseMessageError {
        ParseMessageError{ _priv: () }
    }
}

impl From<num::ParseFloatError> for ParseMessageError {
    fn from(err: num::ParseFloatError) -> ParseMessageError {
        ParseMessageError{ _priv: () }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum StatKind {
    Counter,
    Timer,
    Gauge,
    Sets,
    Histogram,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StatMsg {
    Inc(StatKind, String, i64, f64),
    Set(StatKind, String, i64, f64),
    Del(StatKind, String),
    Bat(Vec<StatMsg>),
}

impl FromStr for StatMsg {
    type Err = ParseMessageError;
    #[inline]
    fn from_str(s: &str) -> Result<StatMsg, ParseMessageError> {
        let msgs: Result<Vec<_>, ParseMessageError> = s.split_terminator('\n')
            .filter(|ss| !ss.is_empty())
            .map(parse_msg)
            .collect();

        match msgs {
            Ok(msg) => {
                if msg.len() == 0 {
                    Err(ParseMessageError { _priv: () })
                } else if msg.len() == 1 {
                    Ok(msg[0].clone())
                } else {
                    Ok(StatMsg::Bat(msg))
                }
            }
            Err(err) => Err(err),
        }
    }
}

fn parse_kind(raw: &str) -> Result<StatKind, ParseMessageError> {
    match raw {
            "c" => Ok(StatKind::Counter),
            "ms" => Ok(StatKind::Timer),
            "g" => Ok(StatKind::Gauge),
            "s" => Ok(StatKind::Sets),
            "h" => Ok(StatKind::Histogram),
            _ => return Err(ParseMessageError { _priv: () })
    }
}

fn build_msg(parts: &Vec<&str>) -> Result<StatMsg, ParseMessageError> {
    let name = parts[0].to_string();
    let kind = try!(parse_kind(parts[2]));

    if parts[1] == "delete" {
        Ok(StatMsg::Del(kind, name))
    }
    else {
        let value = try!(parts[1].parse());
        let sr:f64 = if parts.len() > 3 && parts[3].starts_with('@') {
            try!(parts[3][1..].parse())
        }
        else {
            1.0
        };

        match kind {
            StatKind::Counter =>
                Ok(StatMsg::Inc(kind, name, value, sr)),
            StatKind::Gauge if parts[1].starts_with('+') || parts[1].starts_with('-') =>
                Ok(StatMsg::Inc(kind, name, value, sr)),
            StatKind::Gauge | StatKind::Timer | StatKind::Sets | StatKind::Histogram =>
                Ok(StatMsg::Set(kind, name, value, sr))
        }
    }
}

fn parse_msg(raw: &str) -> Result<StatMsg, ParseMessageError> {
    let parts: Vec<&str> = raw.split(|c| c == ':' || c == '|')
        .filter(|ss| !ss.is_empty())
        .collect();

    match parts.len() {
        0 => Err(ParseMessageError { _priv: () }),
        1 => Ok(StatMsg::Inc(StatKind::Counter, parts[0].to_string(), 1, 1.0)),
        2 => {
            let name = parts[0].to_string();
            match parse_kind(parts[1]) {
                Err(err) =>
                    if let Ok(value) = parts[1].parse() {
                        Ok(StatMsg::Inc(StatKind::Counter, name, value, 1.0))
                    }
                    else { Err(err) },
                Ok(StatKind::Counter) =>
                    Ok(StatMsg::Inc(StatKind::Counter, name, 1, 1.0)),
                Ok(StatKind::Gauge) =>
                    Ok(StatMsg::Inc(StatKind::Gauge, name, 0, 1.0)),
                Ok(kind) =>
                    Ok(StatMsg::Set(kind, name, 0, 1.0)),
            }
        },
        _ => build_msg(&parts),
    }
}

#[test]
fn test_counter() {
    let actual: StatMsg = "test.key:1|c".parse().unwrap();
    let expected = StatMsg::Inc(StatKind::Counter, "test.key".to_string(), 1, 1.0);
    assert_eq!(actual, expected);
}

#[test]
fn test_counter_with_rate() {
    let actual: StatMsg = "test.key:123|c|@0.5".parse().unwrap();
    let expected = StatMsg::Inc(StatKind::Counter, "test.key".to_string(), 123, 0.5);
    assert_eq!(actual, expected);
}

fn test_timer() {
    let actual: StatMsg = "test.key:123|ms".parse().unwrap();
    let expected = StatMsg::Set(StatKind::Timer, "test.key".to_string(), 123, 1.0);
    assert_eq!(actual, expected);
}

#[test]
fn test_timer_with_rate() {
    let actual: StatMsg = "test.key:123|ms|@0.5".parse().unwrap();
    let expected = StatMsg::Set(StatKind::Timer, "test.key".to_string(), 123, 0.5);
    assert_eq!(actual, expected);
}

#[test]
fn test_gauge_absolute() {
    let actual: StatMsg = "test.key:123|g".parse().unwrap();
    let expected = StatMsg::Set(StatKind::Gauge, "test.key".to_string(), 123, 1.0);
    assert_eq!(actual, expected);
}

#[test]
fn test_gauge_delta_pos() {
    let actual: StatMsg = "test.key:+23|g".parse().unwrap();
    let expected = StatMsg::Inc(StatKind::Gauge, "test.key".to_string(), 23, 1.0);
    assert_eq!(actual, expected);
}

#[test]
fn test_gauge_delta_neg() {
    let actual: StatMsg = "test.key:-99|g".parse().unwrap();
    let expected = StatMsg::Inc(StatKind::Gauge, "test.key".to_string(), -99, 1.0);
    assert_eq!(actual, expected);
}

#[test]
fn test_sets() {
    let actual: StatMsg = "test.key:99|s".parse().unwrap();
    let expected = StatMsg::Set(StatKind::Sets, "test.key".to_string(), 99, 1.0);
    assert_eq!(actual, expected);
}

#[test]
fn test_delete() {
    let actual: StatMsg = "test.key:delete|s".parse().unwrap();
    let expected = StatMsg::Del(StatKind::Sets, "test.key".to_string());
    assert_eq!(actual, expected);
}

#[test]
#[should_panic]
fn test_fail_delete_no_type() {
    let actual: StatMsg = "test.key:delete".parse().unwrap();
}

#[test]
fn test_multi() {
    let actual: StatMsg = "stats.counters.test:123|c|@0.5\n\
                            stats.timers.test:123|ms|@0.75\n\
                            stats.gauges.test:123|g\n\
                            stats.gauges.test:-23|g\n\
                            stats.sets.test:99|s\n"
            .parse()
            .unwrap();
    let expected = StatMsg::Bat(vec![
        StatMsg::Inc(StatKind::Counter, "stats.counters.test".to_string(), 123, 0.5),
        StatMsg::Set(StatKind::Timer, "stats.timers.test".to_string(), 123, 0.75),
        StatMsg::Set(StatKind::Gauge, "stats.gauges.test".to_string(), 123, 1.0),
        StatMsg::Inc(StatKind::Gauge, "stats.gauges.test".to_string(), -23, 1.0),
        StatMsg::Set(StatKind::Sets, "stats.sets.test".to_string(), 99, 1.0),
    ]);
    assert_eq!(actual, expected);
}

#[test]
fn test_only_name() {
    let actual: StatMsg = "test.key".parse().unwrap();
    let expected = StatMsg::Inc(StatKind::Counter, "test.key".to_string(), 1, 1.0);
    assert_eq!(actual, expected);
}

#[test]
fn test_no_type() {
    let actual: StatMsg = "test.key:5".parse().unwrap();
    let expected = StatMsg::Inc(StatKind::Counter, "test.key".to_string(), 5, 1.0);
    assert_eq!(actual, expected);
}

#[test]
#[should_panic]
fn test_fail_unknown_type() {
    let actual: StatMsg = "test.key:1|x".parse().unwrap();
}

#[test]
fn test_no_value() {
    let actual: StatMsg = "test.key:|g".parse().unwrap();
    let expected = StatMsg::Inc(StatKind::Gauge, "test.key".to_string(), 0, 1.0);
    assert_eq!(actual, expected);
}

// Metric Types
// $KEY = metric name/bucket
// $PCT = configured percentile thresholds for timer metrics
// Counters = $KEY:1|c
//            $KEY:1|c|@0.1
// Timers   = $KEY:320|ms|@0.1
// Gauges   = $KEY:333|g
//            $KEY:-10|g
//            $KEY:+4|g
// Sets     = $KEY:765|s
// Multi    = gorets:1|c\nglork:320|ms\ngaugor:333|g\nuniques:765|s
//
// Configuration
// config : {
// deleteCounters,
// percentThreshold,
// histogram,
// backends : [graphite, console, repeater],
// graphite : {
// legacyNamespace:  use the legacy namespace [default: true]
// globalPrefix:     global prefix to use for sending stats to graphite [default: "stats"]
// prefixCounter:    graphite prefix for counter metrics [default: "counters"]
// prefixTimer:      graphite prefix for timer metrics [default: "timers"]
// prefixGauge:      graphite prefix for gauge metrics [default: "gauges"]
// prefixSet:        graphite prefix for set metrics [default: "sets"]
// }
// }
//
// Derivations
// Times should generate
// stats.timers.$KEY.mean_$PCT
// stats.timers.$KEY.upper_$PCT
// stats.timers.$KEY.sum_$PCT
//
