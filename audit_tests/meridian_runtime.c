
#include <stdio.h>
#include <stdint.h>
void print_f64(double val) {
    printf("%g\n", val);
}

void print_i64(int64_t val) {
    printf("%lld\n", (long long)val);
}

int main(int argc, char** argv) {
    extern int meridian_main();
    return meridian_main();
}
