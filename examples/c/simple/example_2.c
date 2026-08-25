/* Example 2: GET a value. */
#include <stdio.h>
#include <stdlib.h>

#include "src/mdis_client.h"

int main(void) {
    MdisClient client = mdis_connect("127.0.0.1", 6411);

    char *token = mdis_get(&client, "token");
    if (!token) {
        fprintf(stderr, "Error: could not reach server\n");
        return 1;
    }

    printf("token: %s\n", token);
    free(token);
    return 0;
}
