cargo run --release -- generate \
  -o temp/hard_1m.txt \
  --coeff 0.01 \
  -n 1000000 \
  -v 80000 \
  --value-t-lower-bound 0.5 \
  -w 120000 \
  --correlation Some

cargo run --release -- generate \
  -o temp/hard_100k.txt \
  --coeff 0.01 \
  -n 100000 \
  -v 80000 \
  --value-t-lower-bound 0.5 \
  -w 120000 \
  --correlation Some

cargo run --release -- generate \
  -o temp/hard_10k.txt \
  --coeff 0.01 \
  -n 10000 \
  -v 80000 \
  --value-t-lower-bound 0.5 \
  -w 120000 \
  --correlation Some

cargo run --release -- generate \
  -o temp/hard_1k.txt \
  --coeff 0.01 \
  -n 1000 \
  -v 80000 \
  --value-t-lower-bound 0.5 \
  -w 120000 \
  --correlation Some


