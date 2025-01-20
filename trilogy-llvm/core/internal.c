#include <execinfo.h>
#include <stdlib.h>
#include <stdio.h>
#include "internal.h"
#include "types.h"

void print_trace() {
    void* buffer[100];
    int count = backtrace(buffer, 100);
    char** trace = backtrace_symbols(buffer, count);
    for (int i = 0; i < count; ++i) {
        fprintf(stderr, "%s\n", trace[i]);
    }
    free(trace);
}

[[noreturn]] void internal_panic(char* msg) {
    fprintf(stderr, "%s", msg);
    print_trace();
    exit(255);
}

[[noreturn]] void rte(char* expected, unsigned char tag) {
    fprintf(stderr, "runtime type error: expected %s but received %s\n", expected, type_name(tag));
    print_trace();
    exit(255);
}

[[noreturn]] void exit_(struct trilogy_value* val) {
    switch (val->tag) {
        case TAG_UNIT: exit(0);
        case TAG_INTEGER: exit(val->payload);
        default:
            rte("number", val->tag);
    }
}
