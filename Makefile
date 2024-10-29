PROJECT_NAME=items

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

docker_login:
	docker login ghcr.io -u $(GITHUB_USER) -p $(GITHUB_TOKEN)

backend_container: docker_login
	cd backend && docker build -t ghcr.io/$(GITHUB_USER)/$(PROJECT_NAME)_backend:latest .

frontend_container: docker_login
	cd frontend && docker build -t ghcr.io/$(GITHUB_USER)/$(PROJECT_NAME)_frontend:latest .

containers: backend_container frontend_container test

publish_containers: containers
	docker push ghcr.io/$(GITHUB_USER)/$(PROJECT_NAME)_backend:latest
	docker push ghcr.io/$(GITHUB_USER)/$(PROJECT_NAME)_frontend:latest

backend_container_tagged: docker_login
	cd backend && docker build -t ghcr.io/$(GITHUB_USER)/$(PROJECT_NAME)_backend:$(DOCKERTAG) .

frontend_container_tagged: docker_login
	cd frontend && docker build -t ghcr.io/$(GITHUB_USER)/$(PROJECT_NAME)_frontend:$(DOCKERTAG) .

publish_containers_tagged: containers
	docker push ghcr.io/$(GITHUB_USER)/$(PROJECT_NAME)_backend:$(DOCKERTAG)
	docker push ghcr.io/$(GITHUB_USER)/$(PROJECT_NAME)_frontend:$(DOCKERTAG)
