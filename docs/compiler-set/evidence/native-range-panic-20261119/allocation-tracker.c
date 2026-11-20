#include <stddef.h>
#include <stdio.h>
#include <stdlib.h>

/* Independent link-time observation of generated-program heap lifetimes.
   Static storage avoids introducing tracked allocations in the observer. */
static struct { void *pointer; size_t size; } live[65536];
void *__real_malloc(size_t);
void *__real_calloc(size_t, size_t);
void *__real_realloc(void *, size_t);
void __real_free(void *);
static void remember(void *pointer, size_t size) {
    if (!pointer) return;
    for (size_t i = 0; i < sizeof live / sizeof live[0]; ++i) if (!live[i].pointer) {
        live[i].pointer = pointer; live[i].size = size; return;
    }
    abort();
}
static void forget(void *pointer) {
    if (!pointer) return;
    for (size_t i = 0; i < sizeof live / sizeof live[0]; ++i) if (live[i].pointer == pointer) {
        live[i].pointer = NULL; live[i].size = 0; return;
    }
}
void *__wrap_malloc(size_t size) { void *p = __real_malloc(size); remember(p, size); return p; }
void *__wrap_calloc(size_t n, size_t size) { void *p = __real_calloc(n, size); remember(p, n * size); return p; }
void *__wrap_realloc(void *old, size_t size) {
    void *p = __real_realloc(old, size);
    if (p || size == 0) { forget(old); remember(p, size); }
    return p;
}
void __wrap_free(void *p) { forget(p); __real_free(p); }
__attribute__((destructor)) static void report(void) {
    size_t count = 0, bytes = 0;
    for (size_t i = 0; i < sizeof live / sizeof live[0]; ++i) if (live[i].pointer) {
        fprintf(stderr, "outstanding allocation: %zu bytes\n", live[i].size);
        ++count; bytes += live[i].size;
    }
    fprintf(stderr, "outstanding total: %zu allocations, %zu bytes\n", count, bytes);
}
