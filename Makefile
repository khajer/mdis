.PHONY: build run test docker-build docker-run

build:
	cargo build --release

run:
	cargo run

test:
	cargo test

docker-build:
	docker build -t mdis .

docker-run:
	docker run -p 6411:6411 mdis
