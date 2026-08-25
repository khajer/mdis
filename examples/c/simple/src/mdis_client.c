#define _POSIX_C_SOURCE 200809L /* strdup */

#include "mdis_client.h"

#include <arpa/inet.h>
#include <ctype.h>
#include <netinet/in.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/socket.h>
#include <unistd.h>

#define MAX_BUFFER_SIZE 4096
#define MAX_PARTS 8

static int write_all(int fd, const char *buf, size_t len) {
    size_t sent = 0;
    while (sent < len) {
        ssize_t n = write(fd, buf + sent, len - sent);
        if (n <= 0) {
            return -1;
        }
        sent += (size_t)n;
    }
    return 0;
}

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

/* Mirrors the Node.js/Python/Ruby/Go/Rust client parsing of the same wire
 * format:
 *   SET success:       OK\r\ninsert completed\r\n
 *   GET found:          OK\r\n\r\n{data}\r\n\r\n
 *   GET not found:       OK\r\n\r\n
 *   error/expired key:   Err\r\n
 *
 * Mutates data in place; returns a freshly malloc'd result. */
static char *parse_response(char *data) {
    char *parts[MAX_PARTS];
    int n = split_crlf(data, parts);
    if (n == 0) {
        return strdup("NO RESPONSE");
    }

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
static char *chunk_encode(const char *data, size_t len, size_t *out_len) {
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

static char *send_command(const MdisClient *client, const char *message, size_t message_len) {
    int fd = socket(AF_INET, SOCK_STREAM, 0);
    if (fd < 0) {
        return NULL;
    }

    struct sockaddr_in addr;
    memset(&addr, 0, sizeof(addr));
    addr.sin_family = AF_INET;
    addr.sin_port = htons((uint16_t)client->port);
    if (inet_pton(AF_INET, client->host, &addr.sin_addr) != 1) {
        close(fd);
        return NULL;
    }

    if (connect(fd, (struct sockaddr *)&addr, sizeof(addr)) != 0) {
        close(fd);
        return NULL;
    }

    if (write_all(fd, message, message_len) != 0) {
        close(fd);
        return NULL;
    }
    shutdown(fd, SHUT_WR);

    size_t cap = MAX_BUFFER_SIZE, len = 0;
    char *buf = malloc(cap);
    if (!buf) {
        close(fd);
        return NULL;
    }

    char chunk[MAX_BUFFER_SIZE];
    ssize_t n;
    while ((n = read(fd, chunk, sizeof(chunk))) > 0) {
        if (len + (size_t)n + 1 > cap) {
            cap = (len + (size_t)n + 1) * 2;
            char *grown = realloc(buf, cap);
            if (!grown) {
                free(buf);
                close(fd);
                return NULL;
            }
            buf = grown;
        }
        memcpy(buf + len, chunk, (size_t)n);
        len += (size_t)n;
    }
    buf[len] = '\0';
    close(fd);

    char *result = parse_response(buf); /* server closes after replying */
    free(buf);
    return result;
}

MdisClient mdis_connect(const char *host, int port) {
    MdisClient client;
    memset(&client, 0, sizeof(client));
    snprintf(client.host, sizeof(client.host), "%s", host);
    client.port = port;
    return client;
}

char *mdis_set(const MdisClient *client, const char *key, const char *value, long duration) {
    size_t value_len = strlen(value);

    char headers[128] = {0};
    if (duration != 0) {
        snprintf(headers, sizeof(headers), "Duration: %ld\r\n", duration);
    }

    char *payload = (char *)value;
    size_t payload_len = value_len;
    int payload_owned = 0;
    if (value_len > MAX_BUFFER_SIZE) {
        strncat(headers, "transfer-encoding: chunked\r\n", sizeof(headers) - strlen(headers) - 1);
        payload = chunk_encode(value, value_len, &payload_len);
        if (!payload) {
            return NULL;
        }
        payload_owned = 1;
    }

    size_t msg_cap = strlen(key) + strlen(headers) + payload_len + 64;
    char *msg = malloc(msg_cap);
    if (!msg) {
        if (payload_owned) {
            free(payload);
        }
        return NULL;
    }

    int written = snprintf(msg, msg_cap, "set %s\r\n%s\r\n", key, headers);
    memcpy(msg + written, payload, payload_len);
    written += (int)payload_len;
    memcpy(msg + written, "\r\n\r\n", 4);
    written += 4;

    char *result = send_command(client, msg, (size_t)written);

    free(msg);
    if (payload_owned) {
        free(payload);
    }
    return result;
}

char *mdis_get(const MdisClient *client, const char *key) {
    char msg[300];
    int len = snprintf(msg, sizeof(msg), "get %s\r\n", key);
    return send_command(client, msg, (size_t)len);
}
