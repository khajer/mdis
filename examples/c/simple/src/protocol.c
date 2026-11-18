#define _POSIX_C_SOURCE 200809L /* strdup */

#include "protocol.h"

#include <ctype.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define MAX_PARTS 8
#define CHUNKED_PREFIX "OK\r\ntransfer-encoding: chunked\r\n\r\n"

/* Splits data (in place, like the other clients' string split) on "\r\n"
 * into up to MAX_PARTS pieces and returns how many were found. */
static int split_crlf(char *data, char *parts[MAX_PARTS]) {
    int count = 0;
    char *p = data;
    while (count < MAX_PARTS) {
        char *sep = strstr(p, "\r\n");
        if (!sep) {
            parts[count++] = p;
            break;
        }
        *sep = '\0';
        parts[count++] = p;
        p = sep + 2;
    }
    return count;
}

/* Decodes a chunked response body back into the original value. Mirrors
 * chunk_encode's framing: "{hex_size}\r\n{chunk}\r\n" repeated, terminated
 * by "0\r\n\r\n". `body` is everything after CHUNKED_PREFIX, of length
 * `len`. Caller owns the returned, NUL-terminated buffer. */
char *mdis_chunk_decode(const char *body, size_t len) {
    char *out = malloc(len + 1);
    if (!out) {
        return NULL;
    }
    size_t out_len = 0;
    size_t offset = 0;

    while (offset < len) {
        const char *nl = memchr(body + offset, '\r', len - offset);
        if (!nl || (size_t)(nl - body) + 1 >= len || nl[1] != '\n') {
            break;
        }
        size_t size_len = (size_t)(nl - (body + offset));

        char size_str[32];
        if (size_len >= sizeof(size_str)) {
            break;
        }
        memcpy(size_str, body + offset, size_len);
        size_str[size_len] = '\0';
        size_t size = (size_t)strtoul(size_str, NULL, 16);
        if (size == 0) {
            break;
        }

        size_t data_start = offset + size_len + 2;
        if (data_start + size > len) {
            break;
        }
        memcpy(out + out_len, body + data_start, size);
        out_len += size;
        offset = data_start + size + 2; /* skip the chunk's trailing \r\n */
    }

    out[out_len] = '\0';
    return out;
}

/* Mirrors the Node.js/Python/Ruby/Go/Rust client parsing of the same wire
 * format:
 *   SET success:        OK\r\ninsert completed\r\n
 *   GET found:           OK\r\n\r\n{data}\r\n\r\n
 *   GET not found:        OK\r\n\r\n
 *   GET found (chunked): OK\r\ntransfer-encoding: chunked\r\n\r\n{chunks}
 *   error/expired key:    Err\r\n
 *
 * Mutates data in place; returns a freshly malloc'd result. */
char *mdis_parse_response(char *data) {
    /* A large value comes back chunk-framed rather than as a plain
     * "OK\r\n\r\n{data}\r\n\r\n" body; the chunk data itself may contain
     * \r\n, so it must be decoded from the raw buffer, not the
     * NUL-split parts used below for the other response shapes. */
    size_t prefix_len = strlen(CHUNKED_PREFIX);
    if (strncmp(data, CHUNKED_PREFIX, prefix_len) == 0) {
        return mdis_chunk_decode(data + prefix_len, strlen(data + prefix_len));
    }

    char *parts[MAX_PARTS];
    int n = split_crlf(data, parts);

    char status[8] = {0};
    size_t i;
    for (i = 0; parts[0][i] != '\0' && i < sizeof(status) - 1; i++) {
        status[i] = (char)tolower((unsigned char)parts[0][i]);
    }

    if (strcmp(status, "ok") == 0) {
        if (n >= 3 && parts[1][0] == '\0') {
            return strdup(parts[2]); /* GET with a value */
        }
        if (n >= 2 && parts[1][0] != '\0') {
            return strdup(parts[1]); /* SET response message */
        }
        return strdup(""); /* GET, key not found */
    }
    if (strcmp(status, "err") == 0) {
        return strdup("Error");
    }
    return strdup("NO RESPONSE");
}

/* Splits data into MAX_BUFFER_SIZE chunks using the server's hex-size
 * chunked transfer encoding: "{hex_size}\r\n{chunk}\r\n" repeated,
 * terminated by "0\r\n\r\n". Caller owns the returned buffer. */
char *mdis_chunk_encode(const char *data, size_t len, size_t *out_len) {
    size_t chunk_count = len / MAX_BUFFER_SIZE + 1;
    size_t cap = len + chunk_count * 32 + 16; /* hex header + CRLFs + terminator */
    char *out = malloc(cap);
    if (!out) {
        return NULL;
    }

    size_t pos = 0, offset = 0;
    while (offset < len) {
        size_t size = len - offset < MAX_BUFFER_SIZE ? len - offset : MAX_BUFFER_SIZE;
        pos += (size_t)snprintf(out + pos, cap - pos, "%zx\r\n", size);
        memcpy(out + pos, data + offset, size);
        pos += size;
        out[pos++] = '\r';
        out[pos++] = '\n';
        offset += size;
    }
    pos += (size_t)snprintf(out + pos, cap - pos, "0\r\n\r\n");

    *out_len = pos;
    return out;
}
