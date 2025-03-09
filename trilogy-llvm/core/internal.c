#include "internal.h"
#include "trilogy_number.h"
#include "types.h"
#include <execinfo.h>
#include <stdio.h>
#include <stdlib.h>

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
    fprintf(
        stderr, "runtime type error: expected %s but received %s\n", expected,
        type_name(tag)
    );
    print_trace();
    exit(255);
}

[[noreturn]] void exit_(trilogy_value* val) {
    switch (val->tag) {
    case TAG_UNIT:
        exit(0);
    case TAG_NUMBER:
        exit(trilogy_number_to_ulong(trilogy_number_assume(val)));
    default:
        rte("number", val->tag);
    }
}

void* malloc_safe(size_t size) {
    if (size == 0) return NULL;
    void* ptr = malloc(size);
    if (ptr == NULL) internal_panic("out of memory\n");
    return ptr;
}

void* calloc_safe(size_t num, size_t size) {
    if (num == 0 || size == 0) return NULL;
    void* ptr = calloc(num, size);
    if (ptr == NULL) internal_panic("out of memory\n");
    return ptr;
}

void* realloc_safe(void* ptr, size_t size) {
    if (size == 0) {
        free(ptr);
        return NULL;
    }
    ptr = realloc(ptr, size);
    if (ptr == NULL) internal_panic("out of memory\n");
    return ptr;
}

void trilogy_unhandled_effect(trilogy_value* effect) {
    internal_panic("unhandled effect caused program to terminate\n");
}

void trilogy_execution_ended() {
    internal_panic("the only remaining execution ended\n");
}
