const Utils = require('./utils')
const {Keyring} = require('@polkadot/api');
const unit = 1000000000000;

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
			await demo_show_all();
		});
	program.parse();
}

async function demo_show_all() {
	let api = await Utils.getApi();
	const all = await api.query.system.account.entries();
	for (const account of all) {
		const accountInfo = account[1].data;
		console.log("%s", accountInfo.free / unit);
	}
	process.exit();
}

async function demo_transfer(keyring) {
	let api = await Utils.getApi();
	let moduleMetadata = await Utils.getModules(api);
	const alice = keyring.addFromUri("//Alice");
	const bob = keyring.addFromUri("//Bob");
	let [a, b] = Utils.waitTx(moduleMetadata);
	await api.tx.balances.transfer(bob.address, 2 * unit).signAndSend(alice, a);
	await b();
	process.exit();
}

main()
