IMAGE_NAME=nuop
VERSION=latest
REGISTRY=ghcr.io/ck3mp3r

tests:
	cargo test

build:
	docker build --debug -f docker/Dockerfile . -t $(REGISTRY)/$(IMAGE_NAME):$(VERSION) 
	kind load docker-image $(REGISTRY)/$(IMAGE_NAME):latest -n nuop 
	
buildx:
	docker buildx create --name mybuilder --use || true
	docker buildx inspect --bootstrap
	docker buildx build -f ./operator/docker/Dockerfile --platform linux/amd64,linux/arm64 -t $(REGISTRY)/$(IMAGE_NAME):$(VERSION) --push ./operator

clean:
	cargo clean

clippy:
	cargo clippy

fmt:
	cargo fmt

crds:
	cargo run --bin generate operator/ > chart/crds/nuop.yaml
