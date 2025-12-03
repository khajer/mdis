# mdis
mdis is a json memory cache server. it is a TCP socket server and keep data on memory.

# how to use 

javascript 
```sh
npm install mdis-client;
```

```javascript 
// initial
import MdisClient from mdis-client;
async function main() {
  try {
    let m = MdisClient.connect((host = "127.0.0.1"), (port = 6411));
    //set
    await m.set("token", "123456");

    //get
    const myToken = await m.get("token");
    console.log(myToken);
    
  } catch (error) {
    console.log(error);
  }
}
```


# example
## test client call by client
```sh 
# set 
node example/nodejs/simple/example_1.js
#/ get
node example/nodejs/simple/example_2.js
```
