/* Example 3: SET with a per-key expiration, then GET after it lapses. */
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>

#include "src/mdis_client.h"

int main(void) {
    MdisClient client = mdis_connect("127.0.0.1", 6411);

    char *resp = mdis_set(&client, "token", "123456", 2);
    if (!resp) {
        fprintf(stderr, "Error: could not reach server\n");
        return 1;
    }
    printf("resp: %s\n", resp);
    free(resp);

    sleep(3);

    char *token = mdis_get(&client, "token");
    if (!token) {
        fprintf(stderr, "Error: could not reach server\n");
        return 1;
    }
    printf("token: %s\n", token);
    free(token);
    return 0;
}
