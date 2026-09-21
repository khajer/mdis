<?php

// Example 3: SET with a per-key expiration, then GET after it lapses.
require_once __DIR__ . "/src/MdisClient.php";

$client = MdisClient::connect("127.0.0.1", 6411);

$resp = $client->set("token", "123456", 2);
echo "resp: {$resp}\n";

sleep(3);

$token = $client->get("token");
echo "token: {$token}\n";
