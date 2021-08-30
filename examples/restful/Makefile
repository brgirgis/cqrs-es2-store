
all:
	make clean

clean:
	rm -rf target
	rm -rf Cargo.lock
	rm -rf *.log
	rm -rf test.db*

postgres:
	cargo run --features with-postgres

mysql:
	cargo run --features with-mysql

mariadb:
	cargo run --features with-mariadb

sqlite:
	cargo run --features with-sqlite

mongodb:
	cargo run --features with-mongodb

redis:
	cargo run --features with-redis
