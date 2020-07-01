readme:
	cargo readme > README.md
	git add README.md
	git commit -m "docs: update README.md"

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

# branch := `git rev-parse --abbrev-ref HEAD`
# last_tag := `git tag | tail -1`
# cargo_ver := `grep version Cargo.toml | head -1 | awk '{print $3}' | sed 's/"//g'`
# pwd := `pwd`

# bump minor version and tag
bump-major:
	#!/usr/bin/env bash
	BRANCH=$(git rev-parse --abrev-ref HEAD)
	LAST_TAG=$(git tag | tail -1)
	CARGO_VER=$(grep version Cargo.toml | head -1 | awk '{print $3}' | sed 's/\"//g')
	test $BRANCH == "master"
	test $LAST_TAG == $CARGO_VER
	cargo readme > README.md
	git add README.md
	changelog-rs --latest >>CHANGELOG.md
	git add CHANGELOG.md
	cargo bump major
	cargo update
	git add Cargo.lock Cargo.toml
	CARGO_VER=$(grep version Cargo.toml | head -1 | awk '{print $3}' | sed 's/\"//g')
	git commit -m "release: $NEW_TAG"
	git tag $NEW_TAG
	git push; git push --tags

# bump minor version and tag
bump-minor:
	#!/usr/bin/env bash
	BRANCH=$(git rev-parse --abrev-ref HEAD)
	LAST_TAG=$(git tag | tail -1)
	CARGO_VER=$(grep version Cargo.toml | head -1 | awk '{print $3}' | sed 's/\"//g')
	test $BRANCH == "master"
	test $LAST_TAG == $CARGO_VER
	cargo readme > README.md
	git add README.md
	changelog-rs --latest >>CHANGELOG.md
	git add CHANGELOG.md
	cargo bump minor
	cargo update
	git add Cargo.lock Cargo.toml
	NEW_TAG=$(grep version Cargo.toml | head -1 | awk '{print $3}' | sed 's/\"//g')
	git commit -m "release: $NEW_TAG"
	git tag $NEW_TAG
	git push; git push --tags

# bump patch version and tag
bump-patch:
	#!/usr/bin/env bash
	BRANCH=$(git rev-parse --abrev-ref HEAD)
	LAST_TAG=$(git tag | tail -1)
	CARGO_VER=$(grep version Cargo.toml | head -1 | awk '{print $3}' | sed 's/\"//g')
	test $BRANCH == "master"
	test $LAST_TAG == $CARGO_VER
	cargo readme > README.md
	git add README.md
	changelog-rs --latest >>CHANGELOG.md
	git add CHANGELOG.md
	cargo bump patch
	cargo update
	git add Cargo.lock Cargo.toml
	NEW_TAG=$(grep version Cargo.toml | head -1 | awk '{print $3}' | sed 's/\"//g')
	git commit -m "release: $NEW_TAG"
	git tag $NEW_TAG
	git push; git push --tags

musl:
	docker run -it --rm \
	-v {{pwd}}:/workdir \
	-v ~/.cargo/git:/root/.cargo/git \
	-v ~/.cargo/registry:/root/.cargo/registry \
	registry.gitlab.com/rust_musl_docker/image:stable-latest \
	cargo build --release --target=x86_64-unknown-linux-musl

