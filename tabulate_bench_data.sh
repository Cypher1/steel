#!/bin/bash

# Get relevant lines
data="$(grep -E "(Benchmarking.*: Analyzing|time:)")"
# Strip line noise
data="$(
  echo "$data" | \
  sed "s/Benchmarking //" | \
  sed "s/: Analyzing//"  | \
  sed "s/^.*time:.*\[/[/"
)"
# Replace mark up with commas
data="$(
  echo "$data" | \
  tr '\n' '~' | \
  sed "s/~\[/,/g" | \
  sed "s/\]~/\n/g" | \
  tr '~' ',' | \
  sed "s/\]//")"

# Split up the timing values
tabulated="$(echo "$data" | sed "s/ \(.s\) / \1,/g" | sed "s/ s / s, /g")"
# echo -en "$tabulated"

kinds="$(echo "$tabulated" | grep -E -o "^[^ ]*" | sort | uniq)"
names="$(echo "$tabulated" | grep -E -o "^[^,]*" | sed "s/^[^ ]* //" | sort | uniq)"

grouped=""
IFS=$'\n'
for name in $names; do
  row="$name"
  for kind in $kinds; do
    comparable="$(echo "$tabulated" | grep -F "$kind $name,")"
    for match_line in $comparable; do
      match_data="$(echo "$match_line" | sed "s/^[^,]*,//")"
      row="$row,$match_data"
    done
  done
  grouped="$grouped\n$row"
done

cols="test"
for kind in $kinds; do
  cols="$cols,$kind min,$kind avg,$kind max"
done
echo -ne "$cols"
echo -ne "$grouped" | sort
