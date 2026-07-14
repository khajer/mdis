from src import MdisClient


def main():
    client = None
    try:
        client = MdisClient.connect("127.0.0.1", 6411)
        my_token = client.get("token")
        print(f"resonse: {my_token}")

    except Exception as error:
        print(f"Error: {error}")

    finally:
        if client:
            client.close()


if __name__ == "__main__":
    main()
