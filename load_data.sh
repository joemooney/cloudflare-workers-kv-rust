#!/bin/bash -xv

put() {
    curl --header "Content-Type: application/json" \
         -X POST \
         -d "$1" \
         'localhost:8787'
}
put '{"id":1, "title":"title1", "body":"loren ipsum"}'
put '{"id":2, "title":"title2", "body":"loren ipsum"}'
put '{"id":3, "title":"title3", "body":"loren ipsum"}'
