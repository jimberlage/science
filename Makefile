# Builds an executable for use in development.  `make` defaults to this command.
dev :
	cargo build

release :
	cargo build --release

# Builds gzipped TAR file for homebrew.
brew :
	make release
	ruby ./homebrew_build.rb
