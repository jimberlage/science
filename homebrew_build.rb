#!/usr/bin/env ruby
#
# This builds the app for homebrew.  Honestly, this was just a lot easier than relying on shell
# scripts.

require 'toml'

BREW_DIR = 'target/brew'

# Grab the package metadata from Cargo.lock.
manifest = TOML.load_file('Cargo.lock')
name = manifest['root']['name']
raise 'Must have a package name!!!!' unless name
version = manifest['root']['version']
raise 'Must have a version!!!!' unless version

# Ensure that we have a target/brew directory.
begin
  Dir.mkdir(BREW_DIR)
rescue Errno::EEXIST
  # If the directory exists, that's fine.
end

# Make gzipped TAR file, like homebrew expects.
status = system("tar -cvzf #{BREW_DIR}/#{name}-#{version}.tgz target/release/#{name}")
raise 'tar command failed' unless status
