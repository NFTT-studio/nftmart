const Utils = require('./utils')
const {Keyring} = require('@polkadot/api');
const {bnToBn} = require('@polkadot/util');
const unit = bnToBn('1000000000000');

async function deposit(api, num_proxies) {
	// const proxy.proxyDepositFactor: BalanceOf
	// 198.0000 NMT
	// const proxy.proxyDepositBase: BalanceOf
	// 63.0000 NMT

	// Read from substrate node after substrate node runtime upgrading.
	// const proxyDepositFactor = bnToBn(198);
	// const proxyDepositBase = bnToBn(63);
	const proxyDepositFactor = bnToBn((await api.consts.proxy.proxyDepositFactor).toString());
	const proxyDepositBase = bnToBn((await api.consts.proxy.proxyDepositBase).toString());

	// pub fn deposit(num_proxies: u32) -> BalanceOf<T> {
	// 		if num_proxies == 0 {
	// 		Zero::zero()
	// 	} else {
	// 		T::ProxyDepositBase::get() + T::ProxyDepositFactor::get() * num_proxies.into()
	// 	}
	// }
	if (num_proxies === bnToBn(0)) {
		return bnToBn(0);
	} else {
		return proxyDepositBase.add(proxyDepositFactor).mul(num_proxies);
	}
}

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
			await demo_add_class_admin(keyring);
		});
	program.parse();
}

async function demo_add_class_admin(keyring) {
	let api = await Utils.getApi();
	let moduleMetadata = await Utils.getModules(api);
	const alice = keyring.addFromUri("//Alice");
	const bob = keyring.addFromUri("//Bob");
	let [a, b] = Utils.waitTx(moduleMetadata);
	const classCount = bnToBn((await api.query.ormlNft.nextClassId()).toString());

	const ownerOfClass0 = '62qUEaQwPx7g4vDz88bN4zMBTFmcwLPYbPsvbBhH2QiqWhfB'
	// make sure `ownerOfClass0` has sufficient balances.
	const balancesNeeded = (await deposit(api, classCount.add(bnToBn(1)))).sub(await deposit(api, classCount));

	if (balancesNeeded > bnToBn(0)) {
		await api.tx.balances.transfer(ownerOfClass0, balancesNeeded).signAndSend(alice, a);
		await b();
	}

	// Add Bob as a new admin.
	[a, b] = Utils.waitTx(moduleMetadata);
	const proxyType = 'Any'
	const addNewAdminCall = api.tx.proxy.addProxy(bob.address, proxyType, 0);
	await api.tx.proxy.proxy(ownerOfClass0, proxyType, addNewAdminCall).signAndSend(alice, a);
	await b();
	process.exit();
}

async function demo_show_class_info() {
	let api = await Utils.getApi();
	const classCount = await api.query.ormlNft.nextClassId();
	console.log("class count: %s", classCount);

	const allClasses = await api.query.ormlNft.classes.entries();
	for(const c of allClasses) {
		let key = c[0];
		const len = key.length;
		key = key.buffer.slice(len - 4, len);
		const classID = new Uint32Array(key)[0];
		let clazz = c[1].toJSON();
		clazz.classID = classID;
		clazz.adminList = await api.query.proxy.proxies(clazz.owner);
		console.log("%s", JSON.stringify(clazz));
	}
	process.exit();
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
