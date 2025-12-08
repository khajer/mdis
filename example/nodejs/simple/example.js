const MdisClient = require('./src/index.js');

async function example() {
  try {
    // Create a client instance
    const client = MdisClient.connect();

    console.log('Setting key1 without expiration...');
    const setResult1 = await client.set('key1', 'value1');
    console.log('Result:', setResult1);

    console.log('\nGetting key1...');
    const getResult1 = await client.get('key1');
    console.log('Result:', getResult1);

    console.log('\nSetting key2 with 60 seconds expiration...');
    const setResult2 = await client.set('key2', 'value2', 60);
    console.log('Result:', setResult2);

    console.log('\nGetting key2...');
    const getResult2 = await client.get('key2');
    console.log('Result:', getResult2);

    console.log('\nTrying to get a non-existent key...');
    const getNonExistent = await client.get('nonexistent');
    console.log('Result:', getNonExistent === '' ? 'Empty (key not found)' : getNonExistent);

  } catch (error) {
    console.error('Error:', error);
  }
}

// Run the example
example();
