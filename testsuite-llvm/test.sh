here=$(cd "$(dirname "$0")"; pwd)
pushd "$here" > /dev/null

timefile="time.debug"

while [ "$#" != "0" ]; do
    arg=$1
    case "${arg}" in
        "-q" | "--quiet")
            quiet=true
            ;;
        "-r" | "--release")
            release=true
            timefile="time.release"
            ;;
        "-n" | "--nobuild")
            nobuild=true
            ;;
        *)
            printf "Unrecognized argument $1"
            exit 1
            ;;
    esac
    shift
done

prefix=${LLVM_SYS_180_PREFIX}
if [ -n "${prefix}" ]; then
    prefix="${prefix}/bin/"
fi

if [ -n "${release}" ]; then
    if [ -z "${nobuild}" ]; then
        cargo build --release
    fi
    trilogy="$here/../target/release/trilogy"
else
    if [ -z "${nobuild}" ]; then
        cargo build
    fi
    trilogy="$here/../target/debug/trilogy"
fi

# NOTE: running the binary once trivially up front as MacOS requires verifying the binary
# before it runs, which would mess up the time measurements on the first test case otherwise.
if [ -n "$quiet" ]; then
    "$trilogy" version > /dev/null
else
    "$trilogy" version
fi

result=0
report=""
for dir in $(ls); do
    if [ -d "${dir}" ]; then
        pushd "${dir}" > /dev/null

        expect_output=""
        expect_exit="0"

        if [ -f spec.json ]; then
            expect_exit=$(jq '.exit // 0' -j < spec.json)
            expect_stdout=$(jq '.output // ""' -j < spec.json)
        fi

        output=""
        exit=""
        fail=""

        command time -o "$timefile" "$trilogy" compile main.tri > main.ll
        if [ "$?" != "0" ]; then
            fail="true"
            printf "Failed to compile Trilogy"
            continue
        fi
        "${prefix}clang" main.ll
        if [ "$?" != "0" ]; then
            fail="true"
            printf "Failed to compile LLVM"
            continue
        fi
        output=$(./a.out 2> /dev/null)
        exit=$?
        if [ "$exit" != "$expect_exit" ]; then
            fail="true"
        fi
        if [ "$output" != "$expect_output" ]; then
            fail="true"
        fi

        if [ -n "$fail" ]; then
            result=1
            printf "\e[0;31mx\e[0m %s\n\tExit code: %d (expected %d)\n" "${dir}" "${exit}" "${expect_exit}"
            if [ "$output" != "$expect_output" ]; then
                if [ -z "$expect_output" ]; then
                    printf "Unexpected output:\n%s\n" "$output"
                else
                    printf "Output:\n%s\n" "$output"
                    printf "Expected:\n%s\n" "${expect_output}"
                fi
            fi
        elif [ -z "$quiet" ]; then
            printf "\e[0;32mo\e[0m %s\n" "${dir}"
        fi

        if [ -n "$entry" ]; then
            report="${report}${entry}\n"
        fi
        popd > /dev/null
    fi
done

printf "%s" "$report"
exit $result
