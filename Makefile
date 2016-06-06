# Builds an executable for use in development.  `make` defaults to this command.
dev :
	cargo build

release :
	cargo build --release
