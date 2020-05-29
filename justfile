# generate README.md
readme:
	cargo readme > README.md

clean:
	cargo clean
	find . -type f -name "*.orig" -exec rm {} \;
	find . -type f -name "*.bk" -exec rm {} \;
	find . -type f -name ".*~" -exec rm {} \;

lint:
	cargo clippy

build:
	cargo build

check:
	cargo check

# check tests for errors
check-test:
	cargo check --tests

# print diff of what fmt would do to the codebase
fmt-check:
	cargo fmt -- --check 

unit-test:
	cargo test -- --test-threads=1 --skip integration

int-test:
	cargo test -- --test-threads=1 --skip unit

all-test:
	cargo test -- --test-threads=1

test TEST="":
	cargo test {{TEST}} -- --test-threads=1
	
# generate coverage report and percentage
cov:
	#!/usr/bin/env bash
	TARGET=$(find target/debug -maxdepth 1 -name "git_lab-*" -executable -type f -exec stat -c '%Y %n' {} \;  | sort -nr | head -1 |cut -f2 -d' ')
	echo unit test binary=$TARGET
	kcov --exclude-pattern=/.cargo,/usr/lib,/cargo --verify target/cov $TARGET --test-threads=1
	COVERAGE=$(grep -Po 'covered":.*?[^\\]"' target/cov/index.js | grep "[0-9]*\.[0-9]" -o)
	echo "Coverage:" $COVERAGE

tarp:
	cargo tarpaulin --locked --output-dir tarp -o Html --ignore-tests -- --test-threads=1
