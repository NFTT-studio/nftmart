#!/usr/bin/env bash
post(){ curl -X POST --header "Content-Type:application/json;charset=utf-8" --data "$1" http://127.0.0.1:$RPC_PORT ; }
post '{"jsonrpc":"2.0","id":1,"method":"author_insertKey","params":["gran","include holiday snap brave almost drift grain list short dust hollow poet//nftmart//grandpa//2","0xb46c28b4f0db186814fe579e63d2e9b7c3dbb6c1f28dfe541a6cc11ccfc5fa3e"]}'
post '{"jsonrpc":"2.0","id":1,"method":"author_insertKey","params":["babe","include holiday snap brave almost drift grain list short dust hollow poet//nftmart//babe//2","0x0478a4baa1b4a9b85470a4070738abf190734a2bb2af77dad6ae5fda182da773"]}'
post '{"jsonrpc":"2.0","id":1,"method":"author_insertKey","params":["imon","include holiday snap brave almost drift grain list short dust hollow poet//nftmart//babe//2","0x0478a4baa1b4a9b85470a4070738abf190734a2bb2af77dad6ae5fda182da773"]}'
post '{"jsonrpc":"2.0","id":1,"method":"author_insertKey","params":["audi","include holiday snap brave almost drift grain list short dust hollow poet//nftmart//babe//2","0x0478a4baa1b4a9b85470a4070738abf190734a2bb2af77dad6ae5fda182da773"]}'
