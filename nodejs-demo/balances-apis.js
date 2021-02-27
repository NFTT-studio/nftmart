const Utils = require('./utils')
const {Keyring} = require('@polkadot/api');
const {bnToBn} = require('@polkadot/util');
const unit = bnToBn('1000000000000');

function main() {
	const ss58Format = 50;
	const keyring = new Keyring({type: 'sr25519', ss58Format});
	const {Command} = require('commander');
	const program = new Command();
	program
		.command('transfer')
		.action(async () => {
			await demo_transfer(keyring);
		});
	program
		.command('show-all')
		.action(async () => {
			await demo_show_all(keyring);
		});
	program.parse();
}

async function demo_show_all(keyring) {
	let api = await Utils.getApi();
	const all = await api.query.system.account.entries();
	for (const account of all) {
		let key = account[0];
		const len = key.length;
		key = key.buffer.slice(len - 32, len);
		const addr = keyring.encodeAddress(new Uint8Array(key));
		let data = account[1].toJSON();
		data.address = addr;
		console.log("%s", JSON.stringify(data));
	}
	process.exit();
}

async function demo_transfer(keyring) {
	let api = await Utils.getApi();
	let moduleMetadata = await Utils.getModules(api);
	const bob = keyring.addFromUri("//Bob");
	const alice = keyring.addFromUri("//Alice");
	let [a, b] = Utils.waitTx(moduleMetadata);
	await api.tx.balances.transfer(alice.address, bnToBn(2).mul(unit)).signAndSend(bob, a);
	await b();
	process.exit();
}

main()
