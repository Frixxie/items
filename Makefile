PROJECT_NAME=prestgaarsbasen

all: test

clean:
	cd backend && cargo clean

backend-lint:
	cd backend && cargo clippy

build:
	cd backend && cargo check --verbose
	cd backend && cargo b --verbose

test: build
	docker compose -f docker-compose.yaml up --wait
	cargo install cargo-nextest
	cd backend && cargo nextest run
	docker compose -f docker-compose.yaml down

docker_builder:
	docker buildx create --name builder --platform linux/amd64,linux/arm64

docker_login:
	docker login ghcr.io -u Frixxie -p $(GITHUB_TOKEN)

container: docker_builder docker_login
	cd backend && docker buildx build -t ghcr.io/frixxie/$(PROJECT_NAME):latest . --platform linux/amd64,linux/arm64 --builder builder --push

container_tagged: docker_builder docker_login
	cd backend && docker buildx build -t ghcr.io/frixxie/$(PROJECT_NAME):$(DOCKERTAG) . --platform linux/amd64,linux/arm64 --builder builder --push
