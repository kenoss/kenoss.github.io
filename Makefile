export ZOLA_VERSION := v0.17.2
export ZOLA_DOCKER_IMAGE := ghcr.io/getzola/zola:$(ZOLA_VERSION)

.PHONY: all
all: build

.PHONY: build
build:
	.phony/build

.PHONY: serve
serve:
	.phony/serve
