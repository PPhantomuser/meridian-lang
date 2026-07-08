#include <string.h>
#include <stdlib.h>

double add_c(double a, double b) {
    return a + b;
}

const char* echo_c(const char* s) {
    size_t len = strlen(s);
    char* ret = malloc(len + 6);
    strcpy(ret, "ECHO ");
    strcat(ret, s);
    return ret;
}
