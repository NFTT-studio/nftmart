#!/usr/bin/env bash
post(){ curl -X POST --header "Content-Type:application/json;charset=utf-8" --data "$1" http://127.0.0.1:$RPC_PORT ; }
post '{"jsonrpc":"2.0","id":1,"method":"author_insertKey","params":["gran","include holiday snap brave almost drift grain list short dust hollow poet//nftmart//grandpa//1","0x184f5672c5f405f12476c29ba35ab22fdf44f4e50d671802cb271f06adb5cb3f"]}'
post '{"jsonrpc":"2.0","id":1,"method":"author_insertKey","params":["babe","include holiday snap brave almost drift grain list short dust hollow poet//nftmart//babe//1","0x2020fdf7ad624a75cb35367c68782984cd28e9d9cb93f37397b34602da766b60"]}'
post '{"jsonrpc":"2.0","id":1,"method":"author_insertKey","params":["imon","include holiday snap brave almost drift grain list short dust hollow poet//nftmart//babe//1","0x2020fdf7ad624a75cb35367c68782984cd28e9d9cb93f37397b34602da766b60"]}'
post '{"jsonrpc":"2.0","id":1,"method":"author_insertKey","params":["audi","include holiday snap brave almost drift grain list short dust hollow poet//nftmart//babe//1","0x2020fdf7ad624a75cb35367c68782984cd28e9d9cb93f37397b34602da766b60"]}'
