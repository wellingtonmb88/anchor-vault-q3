
run-tests:
	cargo test --features test-sbf test_initialize
	cargo test --features test-sbf test_deposit
	cargo test --features test-sbf test_withdraw
	cargo test --features test-sbf test_close
