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
let m = MdisClient.connect(localhost='localhost', port=6411)

// string data 
m.set('token', '123456');

// get data
const myToken = m.get('token');

// json data

// set data
const jsData = {
  'username':'khajer',
  'email': 'khajer@gmail.com'
}
m.Json.set("profile", jsData);

// get data
const data = m.Json.get("profile")

// close connection
m.close()

```


# example
## test client call
```sh 
// set 
node example/nodejs/simple/example_1.js
// get
node example/nodejs/simple/example_2.js
```
