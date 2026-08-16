<?php

// Example 4: SET a payload larger than 4096 bytes to exercise chunked
// transfer encoding.
require_once __DIR__ . "/src/MdisClient.php";

$client = MdisClient::connect("127.0.0.1", 6411);
$txtData = str_repeat("a", 5000);

$resp = $client->set("token", $txtData);
echo "{$resp}\n";
