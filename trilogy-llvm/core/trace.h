#pragma once

#ifdef TRILOGY_CORE_TRACE
#include <stdio.h>
#define TRACE(...) fprintf(stderr, __VA_ARGS__);
#else
#define TRACE(...) ((void)0);
#endif
