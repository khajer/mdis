#define _POSIX_C_SOURCE 200809L /* strdup */

#include "mdis_client.h"
#include "protocol.h"

#include <arpa/inet.h>
#include <netinet/in.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/socket.h>
#include <unistd.h>

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

    char *result = mdis_parse_response(buf); /* server closes after replying */
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
        payload = mdis_chunk_encode(value, value_len, &payload_len);
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
