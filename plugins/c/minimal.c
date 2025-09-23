#include <stdio.h>
#include <string.h>

// Plugin ABI = 1
__attribute__((visibility("default")))
const char* plugin_get_meta() {
    return "{\"name\":\"minimal_c\",\"version\":\"0.1.0\",\"abi\":1}";
}

__attribute__((visibility("default")))
int plugin_run(const char* input) {
    printf("[C Plugin] Input: %s\n", input);
    return 0; // success
}