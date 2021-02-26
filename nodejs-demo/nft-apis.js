const Utils = require('./utils')
const {Keyring} = require('@polkadot/api');
const {bnToBn} = require('@polkadot/util');
const unit = bnToBn('1000000000000');

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
	program
		.command('show-class-info')
		.action(async () => {
			await demo_show_class_info();
		});
	program
		.command('add-class-admin')
		.action(async () => {
			await demo_add_class_admin();
		});
	program.parse();
}

async function deposit(num_proxies) {
	// const proxy.proxyDepositFactor: BalanceOf
	// 198.0000 NMT
	// const proxy.proxyDepositBase: BalanceOf
	// 63.0000 NMT

	// Read from substrate node after substrate node runtime upgrading.
	// const proxyDepositFactor = bnToBn(198);
	// const proxyDepositBase = bnToBn(63);
	const proxyDepositFactor = bnToBn(await api.consts.proxy.proxyDepositFactor.toNumber());
	const proxyDepositBase = bnToBn(await api.consts.proxy.proxyDepositBase.toNumber());

	// pub fn deposit(num_proxies: u32) -> BalanceOf<T> {
	// 		if num_proxies == 0 {
	// 		Zero::zero()
	// 	} else {
	// 		T::ProxyDepositBase::get() + T::ProxyDepositFactor::get() * num_proxies.into()
	// 	}
	// }
	if (num_proxies === 0) {
		return bnToBn(0);
	} else {
		return proxyDepositBase.add(proxyDepositFactor).mul(bnToBn(num_proxies));
	}
}

async function demo_add_class_admin() {
	let api = await Utils.getApi();
	let moduleMetadata = await Utils.getModules(api);
	const alice = keyring.addFromUri("//Alice");
	const bob = keyring.addFromUri("//Bob");
	let [a, b] = Utils.waitTx(moduleMetadata);

	const ownerOfClass0 = '62qUEaQwPx7g4vDz88bN4zMBTFmcwLPYbPsvbBhH2QiqWhfB'

	// make sure `ownerOfClass0` has sufficient balances.
	const balancesNeeded = (await deposit(2)).sub(await deposit(1));
	if (balancesNeeded > 0) {
		await api.tx.balances.transfer(ownerOfClass0, balancesNeeded).signAndSend(alice, a);
		await b();
	}

	// Add Bob as a new admin.
	const proxyType = 'Any'
	const addNewAdmin = api.tx.proxy.addProxy(bob.address, proxyType, 0);
	await api.tx.proxy.proxy(ownerOfClass0, proxyType, addNewAdmin).signAndSend(alice, a);
	await b();
	process.exit();
}

async function demo_show_class_info() {
	let api = await Utils.getApi();
	// Query the info of the first class.
	const class0 = await api.query.ormlNft.classes(0);
	// ormlNft.classes: Option<ClassInfoOf>
	// {
	// 	metadata: 0x11,
	// 	totalIssuance: 0,
	// 	owner: 62qUEaQwPx7g4vDz88bN4zMBTFmcwLPYbPsvbBhH2QiqWhfB,
	// 	data: {
	// 		deposit: 0,
	// 		properties: 0,
	// 		name: 11,
	// 		description: 11
	// 	}
	// }
	const adminListOfClass0 = await api.query.proxy.proxies(class0.owner);
	// proxy.proxies: (Vec<ProxyDefinition>,BalanceOf)
	// [
	// 	[
	// 		{
	// 			delegate: 63b4iSPL2bXW7Z1ByBgf65is99LMDLvePLzF4Vd7S96zPYnw,
	// 			proxyType: Any,
	// 			delay: 0
	// 		},
	// 		{
	// 			delegate: 65ADzWZUAKXQGZVhQ7ebqRdqEzMEftKytB8a7rknW82EASXB,
	// 			proxyType: Any,
	// 			delay: 0
	// 		}
	// 	],
	// 	459.0000 NMT
	// ]
}

async function demo_create_class(keyring) {
	let api = await Utils.getApi();
	let moduleMetadata = await Utils.getModules(api);
	const alice = keyring.addFromUri("//Alice");
	let [a, b] = Utils.waitTx(moduleMetadata);
	// pub enum ClassProperty {
	// 	/// Token can be transferred
	// 	Transferable = 0b00000001,
	// 	/// Token can be burned
	// 	Burnable = 0b00000010,
	// }
	await api.tx.nftmart.createClass("https://xx.com/aa.jpg", "aaa", "bbbb", 1 & 2).signAndSend(alice, a);
	await b();
	process.exit();
}

main()
