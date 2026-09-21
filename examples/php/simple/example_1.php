<?php

// Example 1: SET a value.
require_once __DIR__ . "/src/MdisClient.php";

$client = MdisClient::connect("127.0.0.1", 6411);
$resp = $client->set("token", "123456");
echo "response: {$resp}\n";
