#include "internal.h"
#include "trilogy_number.h"
#include "trilogy_string.h"
#include "types.h"
#include <stdio.h>
#include <stdlib.h>

[[noreturn]] void internal_panic(char* msg) {
    fprintf(stderr, "%s", msg);
    exit(255);
}

[[noreturn]] void rte(char* expected, unsigned char tag) {
    fprintf(
        stderr, "runtime type error: expected %s but received %s\n", expected,
        type_name(tag)
    );
    exit(255);
}

[[noreturn]] void exit_(trilogy_value* val) {
    switch (val->tag) {
    case TAG_UNIT:
        exit(0);
    case TAG_NUMBER:
        exit(trilogy_number_to_u64(trilogy_number_assume(val)));
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

static char* unhandled_effect_prefix = "unhandled effect: ";

void trilogy_unhandled_effect(trilogy_value* effect_string) {
    trilogy_string_value* string = trilogy_string_assume(effect_string);
    char* c_string = trilogy_string_as_c(string);
    fprintf(stderr, "%s%s\n", unhandled_effect_prefix, c_string);
    exit(1);
}

void trilogy_execution_ended() {
    internal_panic("the only remaining execution ended\n");
}
