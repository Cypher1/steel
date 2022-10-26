#!/bin/bash
cat out | ./tabulate_bench_data.sh | sed "s/optimize known program //" | sort -n | sed "s/^/optimize known program /" > tabulated
cat tabulated | column -t -s ','
