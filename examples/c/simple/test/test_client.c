/* Whitebox test for the internal parse_response/chunk_encode helpers.
 * Includes the .c file directly to reach its static functions — no test
 * framework, just assert(). */
#include <assert.h>
#include <stdio.h>
#include <string.h>

#include "../src/mdis_client.c"

static void test_parse_response(void) {
    char a[] = "OK\r\ninsert completed\r\n";
    char *r = parse_response(a);
    assert(strcmp(r, "insert completed") == 0);
    free(r);

    char b[] = "OK\r\n\r\nvalue1\r\n\r\n";
    r = parse_response(b);
    assert(strcmp(r, "value1") == 0);
    free(r);

    char c[] = "OK\r\n\r\n";
    r = parse_response(c);
    assert(strcmp(r, "") == 0);
    free(r);

    char d[] = "Err\r\n";
    r = parse_response(d);
    assert(strcmp(r, "Error") == 0);
    free(r);

    char e[] = "garbage";
    r = parse_response(e);
    assert(strcmp(r, "NO RESPONSE") == 0);
    free(r);
}

static void test_parse_response_chunked(void) {
    size_t len = MAX_BUFFER_SIZE + 100;
    char *data = malloc(len + 1);
    memset(data, 'a', len);
    data[len] = '\0';

    size_t encoded_len;
    char *encoded = chunk_encode(data, len, &encoded_len);
    assert(encoded != NULL);

    const char *prefix = "OK\r\ntransfer-encoding: chunked\r\n\r\n";
    char *response = malloc(strlen(prefix) + encoded_len + 1);
    memcpy(response, prefix, strlen(prefix));
    memcpy(response + strlen(prefix), encoded, encoded_len);
    response[strlen(prefix) + encoded_len] = '\0';

    char *r = parse_response(response);
    assert(strlen(r) == len);
    assert(memcmp(r, data, len) == 0);

    free(data);
    free(encoded);
    free(response);
    free(r);
}

static void test_chunk_round_trip(void) {
    size_t len = MAX_BUFFER_SIZE * 2 + 10;
    char *data = malloc(len + 1);
    memset(data, 'a', len);
    data[len] = '\0';

    size_t encoded_len;
    char *encoded = chunk_encode(data, len, &encoded_len);
    assert(encoded != NULL);

    /* Decode using the same rules the server uses and check we get the
     * original bytes back. */
    char *decoded = malloc(len + 1);
    size_t decoded_len = 0;
    char *rest = encoded;
    for (;;) {
        char *nl = strstr(rest, "\r\n");
        assert(nl != NULL);
        long size = strtol(rest, NULL, 16);
        rest = nl + 2;
        if (size == 0) {
            break;
        }
        memcpy(decoded + decoded_len, rest, (size_t)size);
        decoded_len += (size_t)size;
        rest += size + 2; /* skip chunk + trailing CRLF */
    }
    decoded[decoded_len] = '\0';

    assert(decoded_len == len);
    assert(memcmp(decoded, data, len) == 0);

    free(data);
    free(decoded);
    free(encoded);
}

int main(void) {
    test_parse_response();
    test_parse_response_chunked();
    test_chunk_round_trip();
    printf("All tests passed.\n");
    return 0;
}
