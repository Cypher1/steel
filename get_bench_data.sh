#!/bin/bash

tabulated="$(
  grep -E "(Benchmarking.*: Analyzing|time:)" | \
  sed "s/Benchmarking //" | \
  sed "s/: Analyzing//"  | \
  sed "s/^.*time:.*\[/[/" | \
  tr '\n' '~' | \
  sed "s/~\[/\t/g" | \
  sed "s/\]~/\n/g" | \
  tr '~' '\t' | \
  sed "s/\]//" | \
  sed "s/ \(.s\) / \1\t/g"
  )"

# echo -en "$tabulated"

grouped=""
seen=""

IFS=$'\n'
for row in $tabulated; do
  name="$(echo "$row" | cut -f 1)"
  kind="$(echo "$name" | grep -E -o "^[^ ]*")"
  name="$(echo "$name" | sed "s/^$kind //")"
  if echo "$seen" | grep -E -q -o "^$name"; then
    continue
  fi
  seen="$seen\n$name"
  row="$name"
  for match_row in $tabulated; do
    match_name="$(echo "$match_row" | cut -f 1)"
    match_kind="$(echo "$match_name" | grep -E -o "^[^ ]*")"
    match_name="$(echo "$match_name" | sed "s/^$match_kind //")"
    if [[ $match_name != $name ]]; then
      continue
    fi
    match_data="$(echo "$match_row" | sed "s/^[^\t]*\t//")"
    row="$row\t$match_kind\t$match_data"
  done
  grouped="$grouped\n$row"
done

echo -ne "$grouped"
