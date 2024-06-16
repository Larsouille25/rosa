DOCFLAGS := --document-private-items

doc:
	cargo doc $(DOCFLAGS)

docopen:
	cargo doc $(DOCFLAGS) --open

WATCHENV := RUST_BACKTRACE=1

watch:
	cargo watch -s "clear && $(WATCHENV) cargo run --bin $(who) $(path)" --no-vcs-ignores

test:
	cargo test

test-all:
	cargo test -- --include-ignored

.PHONY: doc docopen
