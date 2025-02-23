set quiet
set shell := ["fish", "-c"]

fmt: fmt-rust fmt-c

fmt-rust:
    cargo fmt

[working-directory: "trilogy-llvm/core"]
fmt-c:
    clang-format -i *.{c,h}

check: check-rust check-c

check-rust:
    cargo clippy

[working-directory: "trilogy-llvm/core"]
check-c:
    ls *.{c,h} | xargs -I _ iwyu -Xiwyu --error _

test: test-rust test-tri

test-rust:
    cargo test

[working-directory: "testsuite-llvm"]
test-tri: 
    ./test.sh
