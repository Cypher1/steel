#!/bin/bash
cat out | ./get_bench_data.sh | column -t -s $'\t' | tee tabulated
