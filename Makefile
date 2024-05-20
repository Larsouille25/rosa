DOCFLAGS := --document-private-items

doc:
	cargo doc $(DOCFLAGS)

docopen:
	cargo doc $(DOCFLAGS) --open

watch:
	cargo watch -s "clear && cargo run $(path)" --no-vcs-ignores

.PHONY: doc docopen
