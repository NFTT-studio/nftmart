const Utils = require('./utils')
const { Keyring } = require('@polkadot/api');
const unit = 1000000000000;

async function main() {
	const ss58Format = 50;
	let api = await Utils.getApi(dest = 'ws://8.136.111.191:9944');
	const keyring = new Keyring({ type: 'sr25519', ss58Format });

}

main().catch(console.error).finally(() => process.exit());
