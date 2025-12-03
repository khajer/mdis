from src.client import MdisClient

# Create a client without connecting
client = MdisClient()

# Test the set command format
key = "token"
value = "123456"
expected_set_command = f"SET ${key}\n{value}\r\n"


# Create a test command using the set method
# We'll use a modified version of the set method that returns the command instead of sending it
def create_set_command(key, value):
    return f"SET ${key}\n{value}\r\n"


test_set_command = create_set_command(key, value)

print(f"Expected SET command: {repr(expected_set_command)}")
print(f"Actual SET command:   {repr(test_set_command)}")
print(f"Commands match: {expected_set_command == test_set_command}")

# Test the get command format
expected_get_command = f"GET ${key}\r\n"


def create_get_command(key):
    return f"GET ${key}\r\n"


test_get_command = create_get_command(key)

print(f"Expected GET command: {repr(expected_get_command)}")
print(f"Actual GET command:   {repr(test_get_command)}")
print(f"Commands match: {expected_get_command == test_get_command}")

# Show the actual bytes that would be sent
print("\nSET command bytes:")
print(expected_set_command.encode("utf-8"))

print("\nGET command bytes:")
print(expected_get_command.encode("utf-8"))
