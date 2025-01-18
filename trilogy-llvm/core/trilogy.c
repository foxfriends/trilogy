#include <stdlib.h>
#include <stdio.h>
#include "trilogy.h"
#include "trilogy_value.h"
#include "trilogy_string.h"

void panic(
    struct trilogy_value* rv,
    struct trilogy_value* val
) {
    internal_panic(trilogy_string_to_c(untag_string(val)));
}

void print(
    struct trilogy_value* rv,
    struct trilogy_value* val
) {
    char* ptr = trilogy_string_to_c(untag_string(val));
    printf("%s", ptr);
    free(ptr);
    *rv = trilogy_unit;
}
