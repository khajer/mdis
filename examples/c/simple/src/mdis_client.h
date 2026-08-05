/* Minimal TCP client for the mdis protocol. POSIX sockets only — no
 * external libraries, same spirit as the Python/Ruby/Go/Rust clients.
 */
#ifndef MDIS_CLIENT_H
#define MDIS_CLIENT_H

typedef struct {
    char host[256];
    int port;
} MdisClient;

/* Configures a client for host:port. Does not connect yet — each Set/Get
 * call opens its own short-lived connection. */
MdisClient mdis_connect(const char *host, int port);

/* Stores value under key. duration is the optional per-key expiration in
 * seconds; 0 uses the server's EXPIRE_TIMEOUT.
 *
 * Returns a malloc'd, NUL-terminated string the caller must free(), or
 * NULL on connection error. */
char *mdis_set(const MdisClient *client, const char *key, const char *value, long duration);

/* Retrieves the value stored under key.
 *
 * Returns a malloc'd, NUL-terminated string the caller must free(), or
 * NULL on connection error. */
char *mdis_get(const MdisClient *client, const char *key);

#endif /* MDIS_CLIENT_H */
