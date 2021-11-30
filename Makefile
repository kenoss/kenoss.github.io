export ZOLA_VERSION := 0.14.0
export ZOLA_DOCKER_IMAGE := balthek/zola:$(ZOLA_VERSION)

.PHONY: all
all: build

.PHONY: build
build:
	.phony/build

.PHONY: serve
serve:
	.phony/serve
