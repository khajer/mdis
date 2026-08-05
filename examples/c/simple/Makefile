CC = cc
CFLAGS = -std=c11 -Wall -Wextra -Wpedantic -O2

SRC = src/mdis_client.c
EXAMPLES = example_1 example_2 example_3 example_4

.PHONY: all test clean

all: $(EXAMPLES)

example_%: example_%.c $(SRC) src/mdis_client.h
	$(CC) $(CFLAGS) -o $@ $< $(SRC)

test: test/test_client.c $(SRC) src/mdis_client.h
	$(CC) $(CFLAGS) -o test/test_client test/test_client.c
	./test/test_client

clean:
	rm -f $(EXAMPLES) test/test_client
