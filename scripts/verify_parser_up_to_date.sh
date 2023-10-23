#! /usr/bin/bash
hash1=$(openssl dgst -sha3-256 ./compiler/src/passes/parse/grammar.lalrpop | cut -d " " -f 2)
hash2=$(sed -n "2 s/\/\/ sha3: //p" ./compiler/src/passes/parse/grammar.rs)
test "$hash1" = "$hash2"
