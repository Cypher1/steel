grep -E "(Benchmarking.*: Analyzing|time:)" | \
  sed "s/Benchmarking //" | \
  sed "s/: Analyzing//" | \
  sed "s/^.*time:[^\[]*//" | \
  tr '\n' '~' | \
  sed "s/~\[/\t/g" | \
  tr '~' '\n' | \
  sed "s/\]//" | \
  sed "s/ \(.s\) / \1\t/g"
