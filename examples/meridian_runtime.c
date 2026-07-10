#include <stdio.h>
void print_f64(double val) {
    printf("%g\n", val);
}

int main(int argc, char** argv) {
    extern int meridian_main();
    return meridian_main();
}
