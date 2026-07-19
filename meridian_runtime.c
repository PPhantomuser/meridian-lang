
#include <stdio.h>
#include <stdint.h>
void print_f64(double val) {
    printf("%g\n", val);
}

void print_i64(int64_t val) {
    printf("%lld\n", (long long)val);
}

void print_str(const char* val) {
    printf("%s\n", val);
}

int main(int argc, char** argv) {
    extern int meridian_main();
    return meridian_main();
}
