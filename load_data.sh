#!/bin/bash -xv

URL='https://workers-kv-from-rust.joemooney.workers.dev'
URL='localhost:8787'

put() {
    curl --header "Content-Type: application/json" \
         -X POST \
         -d "$1" \
         $URL
}
put '{"id":1, "title":"title1", "body":"loren ipsum"}'
put '{"id":2, "title":"title2", "body":"loren ipsum"}'
put '{"id":3, "title":"title3", "body":"loren ipsum"}'
