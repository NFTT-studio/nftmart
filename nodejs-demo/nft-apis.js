const Utils = require('./utils')
const {Keyring} = require('@polkadot/api');
const {bnToBn} = require('@polkadot/util');
const unit = bnToBn('1000000000000');

async function nftDeposit(api, metadata, nft_quantity) {
	const createTokenDeposit = bnToBn((await api.consts.nftmart.createTokenDeposit).toString());
	const metaDataByteDeposit = bnToBn((await api.consts.nftmart.metaDataByteDeposit).toString());
	const deposit = createTokenDeposit.add(metaDataByteDeposit.mul(bnToBn(metadata.length)));
	return deposit.mul(nft_quantity);
}

async function proxyDeposit(api, num_proxies) {
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
	program.command('create-class').action(async () => {
		await demo_create_class(keyring);
	});
	program.command('show-class-info').action(async () => {
		await demo_show_class_info();
	});
	program.command('add-class-admin').action(async () => {
		await demo_add_class_admin(keyring);
	});
	program.command('mint-nft <classID>').action(async (classID) => {
		await demo_mint_nft(keyring, classID);
	});
	program.command('show-nft <classID>').action(async (classID) => {
		await demo_show_nft(classID);
	});
	program.parse();
}

async function demo_show_nft(classID) {
	let api = await Utils.getApi();
	const tokenCount = await api.query.ormlNft.nextTokenId(classID);
	console.log(`The token count of class ${classID} is ${tokenCount}.`);
	let classInfo = await api.query.ormlNft.classes(classID);
	if (classInfo.isSome) {
		classInfo = classInfo.unwrap();
		const accountInfo = await api.query.system.account(classInfo.owner);
		console.log(classInfo.toString());
		console.log(accountInfo.toString());
		for (let i = 0; i < tokenCount; i++) {
			let nft = await api.query.ormlNft.tokens(classID, i);
			if (nft.isSome) {
				nft = nft.unwrap();
				console.log(nft.toString());
			}
		}
	}
	process.exit();
}

async function demo_mint_nft(keyring, classID) {
	let api = await Utils.getApi();
	let moduleMetadata = await Utils.getModules(api);
	const alice = keyring.addFromUri("//Alice");
	const classInfo = await api.query.ormlNft.classes(classID);
	if (classInfo.isSome) {
		const ownerOfClass = classInfo.unwrap().owner.toString();
		const nftMetadata = 'aabbccdd';
		const quantity = 3;
		const balancesNeeded = await nftDeposit(api, nftMetadata, bnToBn(quantity));
		const txs = [
			// make sure `ownerOfClass0` has sufficient balances to mint nft.
			api.tx.balances.transfer(ownerOfClass, balancesNeeded),
			// mint nft.
			api.tx.proxy.proxy(ownerOfClass, null, api.tx.nftmart.mint(alice.address, classID, nftMetadata, quantity)),
		];
		const batchExtrinsic = api.tx.utility.batchAll(txs);
		const feeInfo = await batchExtrinsic.paymentInfo(alice);
		console.log("fee of batchExtrinsic: %s", feeInfo.partialFee / unit);

		let [a, b] = Utils.waitTx(moduleMetadata);
		await batchExtrinsic.signAndSend(alice, a);
		await b();
	}
	process.exit();
}

async function demo_add_class_admin(keyring) {
	let api = await Utils.getApi();
	let moduleMetadata = await Utils.getModules(api);
	const alice = keyring.addFromUri("//Alice");
	const bob = keyring.addFromUri("//Bob");
	const classCount = bnToBn((await api.query.ormlNft.nextClassId()).toString());

	const ownerOfClass0 = '62qUEaQwPx7g4vDz88bN4zMBTFmcwLPYbPsvbBhH2QiqWhfB'
	const balancesNeeded = (await proxyDeposit(api, classCount.add(bnToBn(1)))).sub(await proxyDeposit(api, classCount));
	const txs = [
		// make sure `ownerOfClass0` has sufficient balances.
		api.tx.balances.transfer(ownerOfClass0, balancesNeeded),
		// Add Bob as a new admin.
		api.tx.proxy.proxy(ownerOfClass0, null, api.tx.proxy.addProxy(bob.address, 'Any', 0)),
	];
	const batchExtrinsic = api.tx.utility.batchAll(txs);
	const feeInfo = await batchExtrinsic.paymentInfo(alice);
	console.log("fee of batchExtrinsic: %s", feeInfo.partialFee / unit);

	let [a, b] = Utils.waitTx(moduleMetadata);
	await batchExtrinsic.signAndSend(alice, a);
	await b();

	process.exit();
}

async function demo_show_class_info() {
	let api = await Utils.getApi();
	const classCount = await api.query.ormlNft.nextClassId();
	console.log("class count: %s", classCount);

	const allClasses = await api.query.ormlNft.classes.entries();
	for (const c of allClasses) {
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
