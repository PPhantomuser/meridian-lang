#include <stdio.h>
void print_f64(double val) {
    printf("%g\n", val);
}

void print_i64(long long val) {
    printf("%lld\n", val);
}

int main(int argc, char** argv) {
    extern int meridian_main();
    return meridian_main();
}
