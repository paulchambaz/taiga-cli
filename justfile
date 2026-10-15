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
  install -Dm644 completions/_taiga {{ prefix }}/share/zsh/site-functions/_taiga
  install -Dm644 completions/taiga.bash {{ prefix }}/share/bash-completion/completions/taiga
  install -Dm644 completions/taiga.fish {{ prefix }}/share/fish/vendor_completions.d/taiga.fish

vhs:
  vhs demo.tape
