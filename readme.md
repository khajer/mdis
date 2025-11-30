# mdis
mdis is a json memory cache server. it is a TCP socket server and keep data on memory.

# how to use 

```javascript 
// initial
import Client from mdis;
m = Client.connect(localhost="localhost", port=6411)

// set data
const jsData = {
  'username':'khajer',
  'email': 'khajer@gmail.com'
}
m.set("profile", jsData);

// get data
const data = m.get("profile")

// close connection
m.close()

```
