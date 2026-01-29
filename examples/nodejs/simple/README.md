
## installation
```bash
npm install mdis-client
```

## usage
```javascript
const MdisClient = require("mdis-client");
async function main() {
  try {
    const mClient = await MdisClient.connect((host = "127.0.0.1"), (port = 6411));

    mClient.set("token", "123456");
    const myToken = mClient.get("token");
    console.log(myToken);

    mClient.close();
  } catch (error) {
    console.log(error);
  }
}
```
