#!/bin/bash
cat out | ./tabulate_bench_data.sh | column -t -s $'\t' > tabulated
cat tabulated
