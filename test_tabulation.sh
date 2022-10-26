#!/bin/bash
cat out | ./tabulate_bench_data.sh | sed "s/optimize known program //" | sort -n | sed "s/^/optimize known program /" > tabulated
cat tabulated | cut -d',' -f 1,3,6 | column -t -s ','
