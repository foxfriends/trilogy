#include "trilogy_module.h"
#include "internal.h"
#include "trilogy_array.h"
#include "trilogy_callable.h"
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
    trilogy_value* tv, size_t len, uint64_t* ids, void** members
) {
    trilogy_module* module = malloc_safe(sizeof(trilogy_module));
    module->rc = 1;
    module->len = len;
    module->member_ids = ids;
    module->members = members;
    module->closure = NO_CLOSURE;
    return trilogy_module_init(tv, module);
}

trilogy_module* trilogy_module_init_new_closure(
    trilogy_value* tv, size_t len, uint64_t* ids, void** members,
    trilogy_value* closure
) {
    assert(closure->tag == TAG_ARRAY);
    trilogy_module* module = malloc_safe(sizeof(trilogy_module));
    module->rc = 1;
    module->len = len;
    module->member_ids = ids;
    module->members = members;
    module->closure = trilogy_array_assume(closure);
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
        // NOTE: module->member_ids and module->members are not destroyed
        // because they are constant global arrays.
        trilogy_array_destroy(module->closure);
        free(module);
    }
}

typedef trilogy_value* (*accessor)(trilogy_value*);
typedef trilogy_value* (*closure_accessor)(trilogy_value*, trilogy_value*);

void trilogy_module_find(
    trilogy_value* tv, trilogy_module* module, uint64_t id
) {
    // NOTE: modules are typically quite small, so linear search is usually
    // going to be just fine, but if someone makes a pathological module we
    // might do much better to binary search this.
    for (size_t i = 0; i < module->len; ++i) {
        if (module->member_ids[i] == id) {
            if (module->closure == NO_CLOSURE) {
                ((accessor)module->members[i])(tv);
            } else {
                trilogy_value* closure = malloc_safe(sizeof(trilogy_value));
                *closure = trilogy_undefined;
                trilogy_array_clone_into(closure, module->closure);
                ((closure_accessor)module->members[i])(tv, closure);
            }
            return;
        }
    }
    return internal_panic("module does not contain requested member\n");
}
