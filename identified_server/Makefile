NAME=identified
VERSION=$(shell git rev-parse HEAD)
SEMVER_VERSION=$(shell grep version Cargo.toml | awk -F"\"" '{print $$2}' | head -n 1)
REPO=crtschin
SHELL := /bin/bash
include ../.env

# If the first argument is "run"...
ifeq (diesel,$(firstword $(MAKECMDGOALS)))
  # use the rest as arguments for "run"
  RUN_ARGS := $(wordlist 2,$(words $(MAKECMDGOALS)),$(MAKECMDGOALS))
  # ...and turn them into do-nothing targets
  $(eval $(RUN_ARGS):;@:)
endif

define WAIT_FOR_POSTGRES
	if hash psql 2> /dev/null; then \
		until [[ $$RETRIES -ge 10 ]] || PGPASSWORD=$(POSTGRES_PASSWORD) psql -h $${POSTGRES_DB_URL} -U $(POSTGRES_USER) -d $(POSTGRES_DB) -c "select 1" > /dev/null ; do \
			echo "$$((RETRIES++))"; \
			sleep 1; \
		done \
	else \
	  sleep 10 ;\
	fi
endef

has_secrets:
	@[[ $(POSTGRES_DB) ]] || (echo "No secrets set"; exit 2)

no_postgres:
	@[ -z "$$(docker ps -q -f ancestor="postgres:9.6")" ] || (echo "db running"; exit 1)

has_postgres:
	@[ -n "$$(docker ps -q -f ancestor="postgres:9.6")" ] || (echo "db not running"; exit 1)

db: has_secrets	no_postgres
	@echo "Starting postgres container"
	docker run --rm -d \
		-p "5432:5432" \
		--name="$(POSTGRES_DB)" \
		--expose 5432 \
		--env-file=../.env \
		-it postgres:9.6
	@sleep 5
	@diesel migration run --database-url=${DATABASE_URL}

test:
	diesel database setup --database-url=${DATABASE_URL_TEST}
	cargo test
	diesel database drop --database-url=${DATABASE_URL_TEST}

stop:
	@docker ps -aq | xargs -r docker rm -f
	@pkill identified || true


.PHONY: diesel
diesel: has_secrets
	@diesel $(RUN_ARGS) --database-url=${DATABASE_URL}

psql: has_secrets
	@PGPASSWORD=$(POSTGRES_PASSWORD) psql -h ${POSTGRES_DB_URL} -U $(POSTGRES_USER) -d $(POSTGRES_DB)

compose: has_secrets
	docker-compose up -d db
	@echo "Waiting for postgres"
	$(WAIT_FOR_POSTGRES)
	docker-compose up -d web
	docker-compose logs web

watch: has_secrets has_postgres
	systemfd --no-pid -s http::3000 -- cargo watch -x run

compile:
	docker build -t $(REPO)/$(NAME):$(VERSION) .
	strip target/x86_64-unknown-linux-musl/release/identified
	mv target/x86_64-unknown-linux-musl/release/identified .

build:
	docker build -t $(REPO)/$(NAME):$(VERSION) .

tag-latest: build
	docker tag $(REPO)/$(NAME):$(VERSION) $(REPO)/$(NAME):latest
	docker push $(REPO)/$(NAME):latest

tag-semver: build
	if curl -sSL https://registry.hub.docker.com/v1/repositories/$(REPO)/$(NAME)/tags | jq -r ".[].name" | grep -q $(SEMVER_VERSION); then \
		echo "Tag $(SEMVER_VERSION) already exists - not publishing" ; \
	else \
		docker tag $(REPO)/$(NAME):$(VERSION) $(REPO)/$(NAME):$(SEMVER_VERSION) ; \
		docker push $(REPO)/$(NAME):$(SEMVER_VERSION) ; \
	fi