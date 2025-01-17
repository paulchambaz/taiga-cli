run *ARGS:
  cargo run {{ ARGS }}

watch *ARGS:
  cargo watch -x run {{ ARGS }}

build:
  cargo build --release

test:
  cargo test

watch-test:
  cargo watch -x test

coverage:
  cargo tarpaulin

fmt:
  cargo fmt

vhs:
  vhs demo.tape
