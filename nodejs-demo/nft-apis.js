const Utils = require('./utils')
const { Keyring } = require('@polkadot/api');
const unit = 1000000000000;

async function main() {
	const ss58Format = 50;
	const keyring = new Keyring({type: 'sr25519', ss58Format});
	const {Command} = require('commander');
	const program = new Command();
	program
		.command('create-class')
		.action(async () => {
			await demo_create_class(keyring);
		});
	program.parse();
}

async function demo_create_class(keyring) {
	let api = await Utils.getApi();
	let moduleMetadata = await Utils.getModules(api);
	const alice = keyring.addFromUri("//Alice");
	let [a, b] = Utils.waitTx(moduleMetadata);
	// pub enum ClassProperty {
	// 	/// Token can be transferred
	// 	Transferable = 0b00000001,
	// 		/// Token can be burned
	// 		Burnable = 0b00000010,
	// }
	await api.tx.nftmart.createClass("https://xx.com/aa.jpg", "aaa", "bbbb", 1&2).signAndSend(alice, a);
	await b();
	process.exit();
}

main()
