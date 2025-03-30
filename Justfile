set quiet
set shell := ["fish", "-c"]

default: run

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

test:
    cargo test

testsuite:
    cargo test --test testsuite

run file="main.tri":
    cargo run -- compile {{file}} > main.ll
    clang main.ll -g -O0 -rdynamic
    ./a.out

trace file="main.tri":
    TRILOGY_CORE_DEFINES=TRILOGY_CORE_TRACE cargo run -- compile {{file}} > main.ll
    clang main.ll -g -O0 -rdynamic
    ./a.out

clean:
    cargo clean > /dev/null 2>&1
    -rm -f a.out main.ll
    -count testsuite/*/{stdout,stderr,a.out,time.*,*.ll,a.out.dSYM} > /dev/null && rm -r testsuite/*/{stdout,stderr,a.out,time.*,*.ll,a.out.dSYM}
    -count trilogy-llvm/core/*.{ll,bc} > /dev/null && rm trilogy-llvm/core/*.{ll,bc}

[working-directory: "spec"]
spec:
    tectonic -X build
