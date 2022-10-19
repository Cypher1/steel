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
tabulated="$(echo "$data" | sed "s/ \(.s\) / \1,/g")"
# echo -en "$tabulated"

kinds="$(echo $tabulated | grep -E -o "^[^ ]*" | sort | uniq)"
grouped=""
seen=""
IFS=$'\n'
for line in $tabulated; do
  line_kind="$(echo "$line" | grep -E -o "^[^ ]*")"
  name="$(echo "$line" | cut -f 1 | sed "s/^$line_kind //")"
  if [[ $seen =~ $name ]]; then
    continue
  fi
  seen="$seen\n$line"
  row="$name"
  for kind in $kinds; do
    comparable="$(echo "$tabulated" | grep -E -o "^$kind $name,")"
    for match_line in $comparable; do
      match_data="$(echo "$match_line" | sed "s/^[^,]*,//")"
      row="$row,$match_data"
    done
    grouped="$grouped\n$row"
  done
done

cols="test"
for kind in $kinds; do
  cols="$cols,$kind min,$kind avg,$kind max"
done
echo -ne "$cols"
echo -ne "$grouped" | sort
