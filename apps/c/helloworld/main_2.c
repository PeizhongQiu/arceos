#include <stdio.h>

int main()
{
    for (long long i = 0; i < 0x7fffffff; ++i) {
        if(i % 100000000 == 0) {
            printf("This is second thread!\n");
        }
    }
    // printf("This is second thread!\n");
    return 0;
}
