<?php

// Example 2: GET a value.
require_once __DIR__ . "/src/MdisClient.php";

$client = MdisClient::connect("127.0.0.1", 6411);
$token = $client->get("token");
echo "token: {$token}\n";
