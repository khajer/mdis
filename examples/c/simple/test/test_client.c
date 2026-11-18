/* Test for the protocol helpers — no test framework, just assert(). */
#include <assert.h>
#include <stdlib.h>
#include <stdio.h>
#include <string.h>

#include "../src/protocol.h"

static void test_mdis_parse_response(void) {
    char a[] = "OK\r\ninsert completed\r\n";
    char *r = mdis_parse_response(a);
    assert(strcmp(r, "insert completed") == 0);
    free(r);

    char b[] = "OK\r\n\r\nvalue1\r\n\r\n";
    r = mdis_parse_response(b);
    assert(strcmp(r, "value1") == 0);
    free(r);

    char c[] = "OK\r\n\r\n";
    r = mdis_parse_response(c);
    assert(strcmp(r, "") == 0);
    free(r);

    char d[] = "Err\r\n";
    r = mdis_parse_response(d);
    assert(strcmp(r, "Error") == 0);
    free(r);

    char e[] = "garbage";
    r = mdis_parse_response(e);
    assert(strcmp(r, "NO RESPONSE") == 0);
    free(r);
}

static void test_parse_response_chunked(void) {
    size_t len = MAX_BUFFER_SIZE + 100;
    char *data = malloc(len + 1);
    memset(data, 'a', len);
    data[len] = '\0';

    size_t encoded_len;
    char *encoded = mdis_chunk_encode(data, len, &encoded_len);
    assert(encoded != NULL);

    const char *prefix = "OK\r\ntransfer-encoding: chunked\r\n\r\n";
    char *response = malloc(strlen(prefix) + encoded_len + 1);
    memcpy(response, prefix, strlen(prefix));
    memcpy(response + strlen(prefix), encoded, encoded_len);
    response[strlen(prefix) + encoded_len] = '\0';

    char *r = mdis_parse_response(response);
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
    char *encoded = mdis_chunk_encode(data, len, &encoded_len);
    assert(encoded != NULL);

    char *decoded = mdis_chunk_decode(encoded, encoded_len);
    assert(decoded != NULL);
    size_t decoded_len = strlen(decoded);

    assert(decoded_len == len);
    assert(memcmp(decoded, data, len) == 0);

    free(data);
    free(decoded);
    free(encoded);
}

int main(void) {
    test_mdis_parse_response();
    test_parse_response_chunked();
    test_chunk_round_trip();
    printf("All tests passed.\n");
    return 0;
}
