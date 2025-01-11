#include <threads.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>

int thread_func(void *arg){
    for (size_t i = 0; i < 5; i++) {
        printf("i = %zu\n", i);
        sleep(1);
    }

    return 0;
}

int main(int argc, char *argv[]){
    thrd_t th;

    // 스레드 생성
    if (thrd_create(&th, thread_func, NULL) != thrd_success) {
        perror("thrd_create");
        return EXIT_FAILURE;
    }

    // 스레드 디태치
    if (thrd_detach(th) != thrd_success) {
        perror("thrd_detach");
        return EXIT_FAILURE;
    }
6776

    sleep(7);

    return EXIT_SUCCESS;
}