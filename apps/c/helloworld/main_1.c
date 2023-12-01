#include <stdio.h>

int main()
{
    for (long long i = 0; i < 0x7fffffff; ++i) {
        if(i % 1000000 == 0) {
            printf("This is first thread!\n");
        }
    }
    
    return 0;
}
