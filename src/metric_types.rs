/* Metric Types
    $KEY = metric name/bucket
    $PCT = configured percentile thresholds for timer metrics
    Counters = $KEY:1|c
               $KEY:1|c|@0.1
    Timers   = $KEY:320|ms|@0.1
    Gauges   = $KEY:333|g
               $KEY:-10|g
               $KEY:+4|g
    Sets     = $KEY:765|s
    Multi    = gorets:1|c\nglork:320|ms\ngaugor:333|g\nuniques:765|s

    Configuration
    config : {
        deleteCounters,
        percentThreshold,
        histogram,
        backends : [graphite, console, repeater],
        graphite : {
            legacyNamespace:  use the legacy namespace [default: true]
            globalPrefix:     global prefix to use for sending stats to graphite [default: "stats"]
            prefixCounter:    graphite prefix for counter metrics [default: "counters"]
            prefixTimer:      graphite prefix for timer metrics [default: "timers"]
            prefixGauge:      graphite prefix for gauge metrics [default: "gauges"]
            prefixSet:        graphite prefix for set metrics [default: "sets"]
        }
    }

    Derivations
    Times should generate
        stats.timers.$KEY.mean_$PCT
        stats.timers.$KEY.upper_$PCT
        stats.timers.$KEY.sum_$PCT
*/