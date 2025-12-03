#!/usr/bin/env python3
"""
Demonstration of the MDIS client protocol format.

This script shows the exact format of the commands sent to the server
when using the MdisClient class.
"""

from src.client import MdisClient


def demo_protocol():
    """Demonstrate the protocol format for SET and GET commands."""

    print("=== MDIS Client Protocol Demonstration ===\n")

    # Show the exact format of SET commands
    key = "token"
    value = "123456"

    set_command = f"SET ${key}\n{value}\r\n"
    print(f"SET Command Format:")
    print(f"  Key: {key}")
    print(f"  Value: {value}")
    print(f"  Raw command: {repr(set_command)}")
    print(f"  Bytes: {set_command.encode('utf-8')}")
    print()

    # Show the exact format of GET commands
    get_command = f"GET ${key}\r\n"
    print(f"GET Command Format:")
    print(f"  Key: {key}")
    print(f"  Raw command: {repr(get_command)}")
    print(f"  Bytes: {get_command.encode('utf-8')}")
    print()

    # Demonstrate with different values
    print("=== Examples with different values ===\n")

    examples = [
        (
            "jwt-token",
            "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiYWRtaW4iOnRydWUsImlhdCI6MTUxNjIzOTAyMn0.KMUFsIDTnFmyG3nMiGM6H9FNFUROf3wh7SmqJp-QV30",
        ),
        ("user-id", "user-12345"),
        ("session-id", "sess-abcde-54321"),
    ]

    for key, value in examples:
        set_cmd = f"SET ${key}\n{value}\r\n"
        get_cmd = f"GET ${key}\r\n"

        print(f"Key: {key}")
        print(f"  SET: {repr(set_cmd)}")
        print(f"  GET: {repr(get_cmd)}")
        print()


if __name__ == "__main__":
    demo_protocol()
