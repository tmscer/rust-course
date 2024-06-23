
setup:
	docker-compose up -d
	cargo install diesel_cli --no-default-features --features postgres
	@sleep 5
	diesel migration run

stop:
	docker-compose down

destroy: stop
	# `stop` doesn't touch volumes
	docker volume rm server_postgres_data

.PHONY: setup
