/* Example 1: SET a value. */
#include <stdio.h>
#include <stdlib.h>

#include "src/mdis_client.h"

int main(void) {
    MdisClient client = mdis_connect("127.0.0.1", 6411);

    char *resp = mdis_set(&client, "token", "123456", 0);
    if (!resp) {
        fprintf(stderr, "Error: could not reach server\n");
        return 1;
    }

    printf("response: %s\n", resp);
    free(resp);
    return 0;
}
