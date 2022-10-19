#!/bin/bash
cat out | ./tabulate_bench_data.sh > tabulated
cat tabulated | column -t -s ','
