#include "trilogy_module.h"
#include "internal.h"
#include "trilogy_value.h"
#include "types.h"
#include <assert.h>
#include <stdint.h>
#include <stdlib.h>

trilogy_module* trilogy_module_init(trilogy_value* tv, trilogy_module* module) {
    tv->tag = TAG_MODULE;
    tv->payload = (uint64_t)module;
    return module;
}

trilogy_module* trilogy_module_init_new(
    trilogy_value* tv, size_t len, uint64_t* ids, trilogy_value* members
) {
    trilogy_module* module = malloc_safe(sizeof(trilogy_module));
    module->rc = 1;
    module->len = len;
    module->member_ids = ids;
    module->members = members;
    return trilogy_module_init(tv, module);
}

trilogy_module*
trilogy_module_clone_into(trilogy_value* tv, trilogy_module* module) {
    assert(module->rc != 0);
    module->rc++;
    return trilogy_module_init(tv, module);
}

trilogy_module* trilogy_module_untag(trilogy_value* val) {
    if (val->tag != TAG_MODULE) rte("module", val->tag);
    return trilogy_module_assume(val);
}

trilogy_module* trilogy_module_assume(trilogy_value* val) {
    assert(val->tag == TAG_MODULE);
    return (trilogy_module*)val->payload;
}

void trilogy_module_destroy(trilogy_module* module) {
    if (--module->rc == 0) {
        free(module->member_ids);
        for (size_t i = 0; i < module->len; ++i) {
            trilogy_value_destroy(&module->members[i]);
        }
        free(module->members);
        free(module);
    }
}

trilogy_value* trilogy_module_find(trilogy_module* module, uint64_t id) {
    // NOTE: modules are typically quite small, so linear search is usually
    // going to be just fine, but if someone makes a pathological module we
    // might do much better to binary search this.
    for (size_t i = 0; i < module->len; ++i) {
        if (module->member_ids[i] == id) {
            return &module->members[i];
        }
    }
    return NULL;
}
