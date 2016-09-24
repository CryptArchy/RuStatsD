use std::io;
use std::result;
use std::str::{from_utf8, FromStr};
use std::time::Duration;
use std::fmt;
use std::vec::Vec;

#[derive(Debug, Clone, PartialEq)]
pub enum Measurement {
    Counter {
        name: String,
        value: i64,
        rate: f64,
    },
    Timer {
        name: String,
        duration: Duration,
        rate: f64,
    },
    GaugeAbs {
        name: String,
        value: i64,
    },
    GaugeDelta {
        name: String,
        value: i64,
    },
    Sets {
        name: String,
        value: i64,
    },
    Multi(Vec<Measurement>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParseMeasurementError {
    _priv: (),
}

impl fmt::Display for ParseMeasurementError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        "provided string was not a properly formated statsd message".fmt(f)
    }
}

impl FromStr for Measurement {
    type Err = ParseMeasurementError;
    #[inline]
    fn from_str(s: &str) -> Result<Measurement, ParseMeasurementError> {
        let msrs: Result<Vec<_>, ParseMeasurementError> = s.split_terminator('\n')
            .filter(|ss| !s.is_empty())
            .map(parse_as_measure)
            .collect();

        match msrs {
            Ok(ms) => {
                if ms.len() == 0 {
                    Err(ParseMeasurementError { _priv: () })
                } else if ms.len() == 1 {
                    let first = ms[0].clone();
                    Ok(first)
                } else {
                    Ok(Measurement::Multi(ms))
                }
            }
            Err(err) => Err(err),
        }
    }
}

fn parse_as_measure(raw: &str) -> Result<Measurement, ParseMeasurementError> {
    let parts: Vec<&str> = raw.split(|c| c == ':' || c == '|' || c == '@')
        .filter(|ss| !ss.is_empty())
        .collect();
    if parts.len() >= 3 {
        match parts[2] {
            "c" => {
                Ok(Measurement::Counter {
                    name: parts[0].to_string(),
                    value: parts[1].parse().unwrap(),
                    rate: if parts.len() > 3 {
                        parts[3].parse().unwrap()
                    } else {
                        1.0
                    },
                })
            }
            "ms" => {
                Ok(Measurement::Timer {
                    name: parts[0].to_string(),
                    duration: Duration::from_millis(parts[1].parse().unwrap()),
                    rate: if parts.len() > 3 {
                        parts[3].parse().unwrap()
                    } else {
                        1.0
                    },
                })
            }
            "g" => {
                if parts[1].starts_with('+') || parts[1].starts_with('-') {
                    Ok(Measurement::GaugeDelta {
                        name: parts[0].to_string(),
                        value: parts[1].parse().unwrap(),
                    })
                } else {
                    Ok(Measurement::GaugeAbs {
                        name: parts[0].to_string(),
                        value: parts[1].parse().unwrap(),
                    })
                }
            }
            "s" => {
                Ok(Measurement::Sets {
                    name: parts[0].to_string(),
                    value: parts[1].parse().unwrap(),
                })
            }
            _ => Err(ParseMeasurementError { _priv: () }),
        }
    } else {
        Err(ParseMeasurementError { _priv: () })
    }
}

#[test]
fn test_counter() {
    let actual: Measurement = "test.key:1|c".parse().unwrap();
    let expected = Measurement::Counter {
        name: "test.key".to_string(),
        value: 1,
        rate: 1.0,
    };
    assert_eq!(actual, expected);
}

#[test]
fn test_counter_with_rate() {
    let actual: Measurement = "test.key:123|c|@0.5".parse().unwrap();
    let expected = Measurement::Counter {
        name: "test.key".to_string(),
        value: 123,
        rate: 0.5,
    };
    assert_eq!(actual, expected);
}

#[test]
fn test_timer() {
    let actual: Measurement = "test.key:123|ms".parse().unwrap();
    let expected = Measurement::Timer {
        name: "test.key".to_string(),
        duration: Duration::from_millis(123),
        rate: 1.0,
    };
    assert_eq!(actual, expected);
}

#[test]
fn test_timer_with_rate() {
    let actual: Measurement = "test.key:123|ms|@0.5".parse().unwrap();
    let expected = Measurement::Timer {
        name: "test.key".to_string(),
        duration: Duration::from_millis(123),
        rate: 0.5,
    };
    assert_eq!(actual, expected);
}

#[test]
fn test_gauge_absolute() {
    let actual: Measurement = "test.key:123|g".parse().unwrap();
    let expected = Measurement::GaugeAbs {
        name: "test.key".to_string(),
        value: 123,
    };
    assert_eq!(actual, expected);
}

#[test]
fn test_gauge_delta_pos() {
    let actual: Measurement = "test.key:+23|g".parse().unwrap();
    let expected = Measurement::GaugeDelta {
        name: "test.key".to_string(),
        value: 23,
    };
    assert_eq!(actual, expected);
}

#[test]
fn test_gauge_delta_neg() {
    let actual: Measurement = "test.key:-99|g".parse().unwrap();
    let expected = Measurement::GaugeDelta {
        name: "test.key".to_string(),
        value: -99,
    };
    assert_eq!(actual, expected);
}

#[test]
fn test_gauge_sets() {
    let actual: Measurement = "test.key:-99|s".parse().unwrap();
    let expected = Measurement::Sets {
        name: "test.key".to_string(),
        value: -99,
    };
    assert_eq!(actual, expected);
}

#[test]
fn test_multi() {
    let actual: Measurement = "stats.counters.test:123|c|@0.5\n\
                               stats.timers.test:123|ms|@0.75\n\
                               stats.gauges.test:123|g\n\
                               stats.gauges.test:-23|g\n\
                               stats.sets.test:-99|s\n"
            .parse()
            .unwrap();
    let expected = Measurement::Multi(vec![
        Measurement::Counter { name:"stats.counters.test".to_string(), value:123, rate:0.5 },
        Measurement::Timer { name:"stats.timers.test".to_string(), duration:Duration::from_millis(123), rate:0.75 },
        Measurement::GaugeAbs { name:"stats.gauges.test".to_string(), value:123 },
        Measurement::GaugeDelta { name:"stats.gauges.test".to_string(), value:-23 },
        Measurement::Sets { name:"stats.sets.test".to_string(), value:-99 },
    ]);
    assert_eq!(actual, expected);
}

#[test]
#[should_panic]
fn test_fail_no_type() {
    let actual: Measurement = "test.key:1".parse().unwrap();
}

#[test]
#[should_panic]
fn test_fail_unknown_type() {
    let actual: Measurement = "test.key:1|x".parse().unwrap();
}

#[test]
#[should_panic]
fn test_fail_no_value() {
    let actual: Measurement = "test.key:|c".parse().unwrap();
}

// Metric Types
// $KEY = metric name/bucket
// $PCT = configured percentile thresholds for timer metrics
// Counters = $KEY:1|c
// $KEY:1|c|@0.1
// Timers   = $KEY:320|ms|@0.1
// Gauges   = $KEY:333|g
// $KEY:-10|g
// $KEY:+4|g
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
