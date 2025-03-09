#pragma once
#include "bigint.h"
#include <stdbool.h>

typedef struct rational {
    bool is_negative;
    bigint num;
    bigint den;
} rational;
