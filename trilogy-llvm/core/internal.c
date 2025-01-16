#include <stdlib.h>
#include <stdio.h>
#include "internal.h"
#include "types.h"

void panic(char* msg) {
    printf("%s", msg);
    exit(255);
}

void rte(char* expected, unsigned char tag) {
    printf("runtime type error: expected %s but received %s\n", expected, type_name(tag));
    exit(255);
}
