/* Example 4: SET a payload larger than 4096 bytes to exercise chunked
 * transfer encoding. */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "src/mdis_client.h"

int main(void) {
    MdisClient client = mdis_connect("127.0.0.1", 6411);

    size_t len = 5000;
    char *txt_data = malloc(len + 1);
    memset(txt_data, 'a', len);
    txt_data[len] = '\0';

    char *resp = mdis_set(&client, "token", txt_data, 0);
    free(txt_data);
    if (!resp) {
        fprintf(stderr, "Error: could not reach server\n");
        return 1;
    }

    printf("%s\n", resp);
    free(resp);
    return 0;
}
