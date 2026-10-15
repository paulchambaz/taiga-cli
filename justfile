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

lint:
  cargo clippy --all-targets -- -D warnings

man:
  scdoc < taiga.1.scd > taiga.1

install prefix=(env_var('HOME') + '/.local'): build man
  install -Dm755 target/release/taiga {{ prefix }}/bin/taiga
  install -Dm644 taiga.1 {{ prefix }}/share/man/man1/taiga.1

vhs:
  vhs demo.tape
