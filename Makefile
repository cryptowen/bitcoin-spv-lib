schema:
	moleculec --language rust --schema-file ./schema/bitcoin-spv.mol > ./contracts/bitcoin-spv-lib/src/types.rs

.PHONY: schema