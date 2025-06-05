#pragma once
#include "types.h"
#include <stdint.h>

trilogy_module* trilogy_module_init(trilogy_value* tv, trilogy_module* module);
trilogy_module* trilogy_module_init_new(
    trilogy_value* tv, size_t len, uint64_t* ids, void** members
);
trilogy_module* trilogy_module_init_new_closure(
    trilogy_value* tv, size_t len, uint64_t* ids, void** members,
    trilogy_value* closure
);

trilogy_module*
trilogy_module_clone_into(trilogy_value* tv, trilogy_module* module);
void trilogy_module_destroy(trilogy_module* module);

trilogy_module* trilogy_module_untag(trilogy_value* val);
trilogy_module* trilogy_module_assume(trilogy_value* val);

void trilogy_module_find(
    trilogy_value* tv, trilogy_module* module, uint64_t id
);
