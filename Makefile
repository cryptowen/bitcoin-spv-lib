schema:
	echo '#![allow(dead_code)]' > tmp_file
	moleculec --language rust --schema-file ./schema/bitcoin-spv.mol >> tmp_file
	mv tmp_file ./contracts/bitcoin-spv-lib/src/types.rs
	cd contracts/bitcoin-spv-lib && cargo fmt
	cp ./contracts/bitcoin-spv-lib/src/types.rs tests/src/types/bitcoin_spv_lib.rs

.PHONY: schema