// Plaintext protocol: <metric path> <metric value> <metric timestamp>\n
// PORT=2003
// SERVER=graphite.your.org
// echo "local.random.diceroll 4 `date +%s`" | nc -c ${SERVER} ${PORT}

// Pickle protocol: [(path, (timestamp, value)), ...]

// Events (only supported by the graphite web app over HTTP)
// $ curl -X POST "http://graphite/events/"
//    -d '{ "what": "Event - deploy", "tags": ["deploy"],
//    "data": "deploy of master branch happened at Wed Jul  6 22:34:41 UTC 2016" }'