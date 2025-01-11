#include <threads.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>

#define NUM_THREADS 10

int thread_func(void *arg) {
    size_t id = (size_t)arg;
    for (size_t i = 0; i < 5; i++) {
        printf("id = %zu, i = %zu\n", id, i);
        sleep(1);
    }

    return 0;
}

int main(int argc, char const *argv[]) {
    thrd_t v[NUM_THREADS];

    for (size_t i = 0; i < NUM_THREADS; i++) {
        if (thrd_create(&v[i], thread_func, (void *)i) != thrd_success) {
            perror("thrd_create");
            return EXIT_FAILURE;
        }
    }

    for (size_t i = 0; i < NUM_THREADS; i++) {
        int ret;

        if (thrd_join(v[i], &ret) == thrd_success) {
            printf("return =%d\n", ret);
        } else {
            perror("thrd_join");
            return EXIT_FAILURE;
        }
    }

    return EXIT_SUCCESS;
}