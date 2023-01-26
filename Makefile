.PHONY: all build-homie build-frontend

PROFILE ?= "dev"

all: build-homie

build-homie: build-frontend
	cargo build --profile ${PROFILE}

build-frontend:
	cd frontend && pnpm install
	cd frontend && pnpm build
