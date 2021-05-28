import {getApi, getModules, waitTx, hexToUtf8, unit} from "./utils.mjs";
import {Keyring} from "@polkadot/api";
import {bnToBn} from "@polkadot/util";
import {Command} from "commander";

async function proxyDeposit(api, num_proxies) {
	try {
		let deposit = await api.ws.call('nftmart_addClassAdminDeposit', [num_proxies], 10000);
		return bnToBn(deposit);
	} catch (e) {
		console.log(e);
		return null;
	}
}

async function main() {
	const ss58Format = 50;
	const keyring = new Keyring({type: 'sr25519', ss58Format});
	const program = new Command();
	program.option('--ws <addr>', 'node ws addr', 'ws://192.168.0.2:9944');

	program.command('create_class <signer>').action(async (signer) => {
		await create_class(program.opts().ws, keyring, signer);
	});
	program.command('add_class_admin <admin> <classId> <newAdmin>').action(async (admin, classId, newAdmin) => {
		await add_class_admin(program.opts().ws, keyring, admin, classId, newAdmin);
	});
	program.command('show_class').action(async () => {
		await show_class(program.opts().ws);
	});
	program.command('show_whitelist').action(async () => {
		await show_whitelist(program.opts().ws, keyring);
	});
	program.command('add_whitelist <sudo> <account>').action(async (sudo, account) => {
		await add_whitelist(program.opts().ws, keyring, sudo, account);
	});

	await program.parseAsync(process.argv);
}

async function add_class_admin(ws, keyring, admin, classId, newAdmin) {
	let api = await getApi(ws);
	let moduleMetadata = await getModules(api);
	admin = keyring.addFromUri(admin);
	newAdmin = keyring.addFromUri(newAdmin);

	let classInfo = await api.query.ormlNft.classes(classId);
	if(classInfo.isSome) {
		classInfo = classInfo.unwrap();
		const ownerOfClass = classInfo.owner;
		console.log(ownerOfClass.toString());
		const balancesNeeded = await proxyDeposit(api, 1);
		if (balancesNeeded === null) {
			console.log("error: await proxyDeposit(api, 1);");
			return;
		}
		console.log("adding a class admin needs to reserve %s NMT", balancesNeeded / unit);
		const txs = [
			// make sure `ownerOfClass` has sufficient balances.
			api.tx.balances.transfer(ownerOfClass, balancesNeeded),
			// Add `newAdmin` as a new admin.
			api.tx.proxy.proxy(ownerOfClass, null, api.tx.proxy.addProxy(newAdmin.address, 'Any', 0)),
			// api.tx.proxy.proxy(ownerOfClass, null, api.tx.proxy.removeProxy(newAdmin.address, 'Any', 0)), to remove an admin
		];
		const batchExtrinsic = api.tx.utility.batchAll(txs);
		const feeInfo = await batchExtrinsic.paymentInfo(admin);
		console.log("fee of batchExtrinsic: %s NMT", feeInfo.partialFee / unit);

		let [a, b] = waitTx(moduleMetadata);
		await batchExtrinsic.signAndSend(admin, a);
		await b();
	}
}

async function show_class(ws) {
	let api = await getApi(ws);
	let classCount = 0;

	const allClasses = await api.query.ormlNft.classes.entries();
	let all = [];
	for (const c of allClasses) {
		let key = c[0];
		const len = key.length;
		key = key.buffer.slice(len - 4, len);
		const classID = new Uint32Array(key)[0];
		let clazz = c[1].toJSON();
		clazz.metadata = hexToUtf8(clazz.metadata.slice(2));
		clazz.classID = classID;
		clazz.adminList = await api.query.proxy.proxies(clazz.owner);
		all.push(JSON.stringify(clazz));
		classCount++;
	}
	console.log("%s", all);
	console.log("class count: %s", classCount);
	console.log("nextClassId: %s", await api.query.ormlNft.nextClassId());
}

async function add_whitelist(ws, keyring, sudo, account) {
	// usage: node nft-apis.mjs add-whitelist //Alice 63dHdZZMdgFeHs544yboqnVvrnAaTRdPWPC1u2aZjpC5HTqx
	let api = await getApi(ws);
	let moduleMetadata = await getModules(api);
	sudo = keyring.addFromUri(sudo);
	if(account.length !== '62qUEaQwPx7g4vDz88cT36XXuEUQmYo3Y5dxnxScsiDkb8wy'.length){
		account = keyring.addFromUri(account);
		account = account.address;
	}
	// const call = api.tx.sudo.sudo(api.tx.config.removeWhitelist(account.address)); to remove account from whitelist
	const call = api.tx.sudo.sudo(api.tx.nftmartConf.addWhitelist(account));
	const feeInfo = await call.paymentInfo(sudo.address);
	console.log("The fee of the call: %s.", feeInfo.partialFee / unit);
	let [a, b] = waitTx(moduleMetadata);
	await call.signAndSend(sudo, a);
	await b();
}

async function show_whitelist(ws, keyring) {
	let api = await getApi(ws);
	const all = await api.query.nftmartConf.accountWhitelist.entries();
	for (const account of all) {
		let key = account[0];
		const len = key.length;
		key = key.buffer.slice(len - 32, len);
		const addr = keyring.encodeAddress(new Uint8Array(key));
		console.log("%s", addr);
	}
}

async function create_class(ws, keyring, signer) {
	let api = await getApi(ws);
	let moduleMetadata = await getModules(api);
	signer = keyring.addFromUri(signer);
	let [a, b] = waitTx(moduleMetadata);
	// 	Transferable = 0b00000001,
	// 	Burnable = 0b00000010,
	// 	RoyaltiesChargeable = 0b00000100,
	await api.tx.nftmart.createClass("https://xx.com/aa.jpg", "aaa", "bbbb", 1 | 2 | 4).signAndSend(signer, a);
	await b();
}

main().then(r => {
	process.exit();
}).catch(err => {
	console.log(err);
	process.exit();
});
