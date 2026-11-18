/* Wire-format helpers for the mdis protocol (no I/O). */
#ifndef MDIS_PROTOCOL_H
#define MDIS_PROTOCOL_H

#include <stddef.h>

#define MAX_BUFFER_SIZE 4096

/* Splits data into MAX_BUFFER_SIZE chunks using the server's hex-size
 * chunked transfer encoding. Returns a malloc'd buffer (length in
 * *out_len) the caller must free(), or NULL on allocation failure. */
char *mdis_chunk_encode(const char *data, size_t len, size_t *out_len);

/* Decodes a chunked response body of length len. Returns a malloc'd,
 * NUL-terminated string the caller must free(), or NULL. */
char *mdis_chunk_decode(const char *body, size_t len);

/* Parses a raw server response. Mutates data in place; returns a freshly
 * malloc'd result the caller must free(). */
char *mdis_parse_response(char *data);

#endif /* MDIS_PROTOCOL_H */
