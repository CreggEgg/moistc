#!/run/current-system/sw/bin/bash
cargo run build ./moistc-example.wet
gcc main.o core.c
./a.out
