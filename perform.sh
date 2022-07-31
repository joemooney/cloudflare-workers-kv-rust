#!/bin/bash

URL='https://workers-kv-from-rust.joemooney.workers.dev'
URL='localhost:8787'

# function to POST a KeyValue
put() {
    curl --header "Content-Type: application/json" \
         -X POST \
         -d "$1" \
         $URL
    rc=$?
    case $rc in
    0) echo "[pass] put $*";;
    *) echo '[fail] if Connection refused => `wrangler dev` not running?';;
    esac
}

# function to get a KeyValue
get()
{
    # echo curl -X GET $URL/kv/$1
    json=$(curl -s -X GET $URL/kv/$1)
    rc=$?
    case $rc in
    0) 
        if [ "$json" = '""' ]; then
            echo "[fail] not found kv:$1";
        else
            echo "[pass] get $*";
            # json=$(curl -s -X GET $URL/kv/$1)
            # we need to unescape into a json string that jq will parse
            eval echo $json | jq .
        fi
    ;;
    7) 
       ps -ef|grep 'wranger de[v]'
       echo '[fail] if Connection refused => check if `wrangler dev` not running?'
    ;;
    *) echo '[fail] get';;
    esac
}
kv() { 
    printf "{ \"key\": $1, \"value\": \"$(echo $2 | sed 's,",\\\\\",g')\" }"; 
}

load() {
    kv1=$(kv 1 '{ "title":"title1", "body":"loren ipsum" }')
    kv2=$(kv 2 '{ "title":"title2", "body":"loren ipsum" }')
    kv3=$(kv 3 '{ "title":"title3", "body":"loren ipsum" }')
    put "$kv1"
    put "$kv2"
    put "$kv3"
}

usage() {
    echo "./perform.sh [put|get|load] ..."
    exit 1
}

cmd=$1; shift

case "$cmd" in 
"put")  put "$@";;
"load")  load;;
"get")  get "$@";;
"" | "-h" | "help" | "--help")  usage;;
*)  usage;
esac
