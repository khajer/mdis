# mdis
mdis is a json memory cache server. it is a TCP socket server and keep data on memory.
 
## how to run server

### docker build 
```sh
# pull code 
git clone https://github.com/yourusername/mdis.git
cd mdis

# build image
docker build -t mdis .

# run image
docker run -p 6411:6411 mdis
```

By default, the data expires after 5 minutes. You can change this timeout by setting the `EXPIRE_TIMEOUT` environment variable.
```sh
docker run -p 6411:6411 mdis env EXPIRE_TIMEOUT=10000
```
## how to use 

javascript 
```sh
npm install mdis-client
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
node examples/nodejs/simple/example_1.js
#/ get
node examples/nodejs/simple/example_2.js
```
