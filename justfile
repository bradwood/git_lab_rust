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

graphql:
	graphql-client introspect-schema https://gitlab.com/api/graphql > src/graphql/schema.json

check:
	cargo check

# check tests for errors
check-test:
	cargo check --tests

# print diff of what fmt would do to the codebase
fmt-check:
	cargo fmt -- --check

unit-tests:
	cargo test config_unit_tests -- --test-threads=1 --skip integration
	cargo test -- --skip config_unit_tests --test-threads=1 --skip integration

int-tests:
	cargo test config_unit_tests -- --test-threads=1 --skip unit
	cargo test -- --skip config_unit_tests --test-threads=1 --skip unit

all-tests:
	cargo test config_unit_tests -- --test-threads=1
	cargo test -- --skip config_unit_tests --test-threads=1

test TEST:
	cargo test {{TEST}} -- --test-threads=1 --show-output

tarp:
	cargo tarpaulin

branch := `git rev-parse --abbrev-ref HEAD`
last_tag := `git tag | tail -1`
cargo_ver := `grep version Cargo.toml | head -1 | awk '{print $3}' | sed 's/"//g'`

# bump minor version and tag
bump-major:
	test {{branch}} == "master"
	test {{last_tag}} == {{cargo_ver}}
	cargo bump major --git-tag

# bump minor version and tag
bump-minor:
	test {{branch}} == "master"
	test {{last_tag}} == {{cargo_ver}}
	cargo bump minor --git-tag

# bump patch version and tag
bump-patch:
	test {{branch}} == "master"
	test {{last_tag}} == {{cargo_ver}}
	cargo bump patch --git-tag
