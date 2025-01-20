#include "internal.h"
#include "trilogy_array.h"
#include "trilogy_boolean.h"
#include "trilogy_string.h"
#include "trilogy_value.h"
#include "types.h"
#include <execinfo.h>
#include <stdio.h>
#include <stdlib.h>

void panic(trilogy_value* rv, trilogy_value* val) {
    internal_panic(trilogy_string_as_c(trilogy_string_untag(val)));
}

void print(trilogy_value* rv, trilogy_value* val) {
    char* ptr = trilogy_string_as_c(trilogy_string_untag(val));
    printf("%s", ptr);
    free(ptr);
    *rv = trilogy_unit;
}

void trace(trilogy_value* rt) {
    void* buffer[100];
    int count = backtrace(buffer, 100);
    trilogy_array_value* arr = trilogy_array_init_cap(rt, count);

    char** trace = backtrace_symbols(buffer, count);
    for (int i = 0; i < count; ++i) {
        trilogy_string_init_from_c(&arr->contents[i], trace[i]);
    }
    free(trace);
}

void referential_eq(trilogy_value* rv, trilogy_value* lhs, trilogy_value* rhs) {
    *rv = trilogy_boolean(trilogy_value_referential_eq(lhs, rhs));
}

void structural_eq(trilogy_value* rv, trilogy_value* lhs, trilogy_value* rhs) {
    *rv = trilogy_boolean(trilogy_value_structural_eq(lhs, rhs));
}
