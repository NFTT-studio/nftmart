import {getApi, getModules, waitTx, hexToUtf8, unit, ensureAddress, u32sToU64} from "./utils.mjs";
import {Keyring} from "@polkadot/api";
import {bnToBn} from "@polkadot/util";
import {Command} from "commander";

async function proxyDeposit(num_proxies) {
	try {
		let deposit = await Global_Api.ws.call('nftmart_addClassAdminDeposit', [num_proxies], 10000);
		return bnToBn(deposit);
	} catch (e) {
		console.log(e);
		return null;
	}
}

async function nftDeposit(metadata) {
	try {
		let depositAll = await Global_Api.ws.call('nftmart_mintTokenDeposit', [metadata.length], 10000);
		return bnToBn(depositAll);
	} catch (e) {
		console.log(e);
		return null;
	}
}

async function classDeposit(metadata, name, description) {
	try {
		let [_deposit, depositAll] = await Global_Api.ws.call('nftmart_createClassDeposit', [metadata.length, name.length, description.length], 10000);
		return bnToBn(depositAll);
	} catch (e) {
		console.log(e);
		return null;
	}
}


let Global_Api = null;
let Global_ModuleMetadata = null;

async function initApi(ws) {
	if (Global_Api === null || Global_ModuleMetadata === null) {
		Global_Api = await getApi(ws);
		Global_ModuleMetadata = await getModules(Global_Api);
	}
}

async function main() {
	const ss58Format = 50;
	const keyring = new Keyring({type: 'sr25519', ss58Format});
	const program = new Command();
	program.option('--ws <url>', 'node ws addr', 'ws://192.168.0.2:9944');

	// node nft-apis.mjs --ws 'ws://81.70.132.13:9944' create_class //Alice
	program.command('create_class <signer>').action(async (signer) => {
		await create_class(program.opts().ws, keyring, signer);
	});
	// 1. node nft-apis.mjs --ws 'ws://81.70.132.13:9944' add_class_admin //Alice 0 //Bob
	// 2. node nft-apis.mjs --ws 'ws://81.70.132.13:9944' add_class_admin //Alice 0 63b4iSPL2bXW7Z1ByBgf65is99LMDLvePLzF4Vd7S96zPYnw
	program.command('add_class_admin <admin> <classId> <newAdmin>').action(async (admin, classId, newAdmin) => {
		await add_class_admin(program.opts().ws, keyring, admin, classId, newAdmin);
	});
	// node nft-apis.mjs --ws 'ws://81.70.132.13:9944' show_class
	program.command('show_class').action(async () => {
		await show_class(program.opts().ws);
	});
	// node nft-apis.mjs --ws 'ws://81.70.132.13:9944' show_whitelist
	program.command('show_whitelist').action(async () => {
		await show_whitelist(program.opts().ws, keyring);
	});
	// 1. node nft-apis.mjs --ws 'ws://81.70.132.13:9944' add_whitelist //Alice //Bob
	// 2. node nft-apis.mjs --ws 'ws://81.70.132.13:9944' add_whitelist //Alice 63b4iSPL2bXW7Z1ByBgf65is99LMDLvePLzF4Vd7S96zPYnw
	program.command('add_whitelist <sudo> <account>').action(async (sudo, account) => {
		await add_whitelist(program.opts().ws, keyring, sudo, account);
	});
	// node nft-apis.mjs --ws 'ws://81.70.132.13:9944' mint_nft //Alice 0 30
	program.command('mint_nft <admin> <classID> <quantity>').action(async (admin, classID, quantity) => {
		await mint_nft(program.opts().ws, keyring, admin, classID, quantity);
	});
	// 1: node nft-apis.mjs --ws 'ws://81.70.132.13:9944' show_nfts
	// 2: node nft-apis.mjs --ws 'ws://81.70.132.13:9944' show_nfts 0
	program.command('show_nft [classID]').action(async (classID) => {
		await show_nft(program.opts().ws, classID);
	});
	// 1. node nft-apis.mjs --ws 'ws://81.70.132.13:9944' query_nft_by //Alice
	// 2. node nft-apis.mjs --ws 'ws://81.70.132.13:9944' query_nft_by 65ADzWZUAKXQGZVhQ7ebqRdqEzMEftKytB8a7rknW82EASXB
	program.command('query_nft_by <account>').action(async (account) => {
		await query_nft_by(program.opts().ws, keyring, account);
	});
	// 1. node nft-apis.mjs --ws 'ws://81.70.132.13:9944' query_class_by //Alice
	// 2. node nft-apis.mjs --ws 'ws://81.70.132.13:9944' query_class_by 65ADzWZUAKXQGZVhQ7ebqRdqEzMEftKytB8a7rknW82EASXB
	program.command('query_class_by <account>').action(async (account) => {
		await query_class_by(program.opts().ws, keyring, account);
	});
	// 1. node nft-apis.mjs --ws 'ws://81.70.132.13:9944' transfer_nfts //Alice 65ADzWZUAKXQGZVhQ7ebqRdqEzMEftKytB8a7rknW82EASXB \
	//		--classId 0 --tokenId 0 --quantity 1 \
	//		--classId 0 --tokenId 1 --quantity 2 \
	//		--classId 0 --tokenId 2 --quantity 3
	// 2. node nft-apis.mjs --ws 'ws://81.70.132.13:9944' transfer_nfts //Alice //Bob \
	//		--classId 0 --tokenId 0 --quantity 1 \
	//		--classId 0 --tokenId 1 --quantity 2 \
	//		--classId 0 --tokenId 2 --quantity 3
	program.command('transfer_nfts <from> <to>')
		.requiredOption('-c, --classId <classIds...>')
		.requiredOption('-t, --tokenId <tokenIds...>')
		.requiredOption('-q, --quantity <quantities...>')
		.action(async (from, to, {classId, tokenId, quantity}) => {
			if(classId.length === tokenId.length && tokenId.length === quantity.length) {
				const tokens = classId.map((e, i) => {
					return [BigInt(e), BigInt(tokenId[i]), BigInt(quantity[i])];
				});
				await transfer_nfts(program.opts().ws, tokens, keyring, from, to);
			} else {
				console.log("Invalid options, maybe the length of classIds mismatches with the length of tokenIds.");
			}
	});
	// node nft-apis.mjs --ws 'ws://81.70.132.13:9944' burn_nft //Alice 0 0 20
	program.command('burn_nft <signer> <classID> <tokenID> <quantity>').action(async (signer, classID, tokenID, quantity) => {
		await burn_nft(program.opts().ws, keyring, signer, classID, tokenID, quantity);
	});
	// node nft-apis.mjs --ws 'ws://81.70.132.13:9944' destroy_class //Alice 0
	program.command('destroy_class <signer> <classID>').action(async (signer, classID) => {
		await destroy_class(program.opts().ws, keyring, signer, classID);
	});
	// node nft-apis.mjs --ws 'ws://81.70.132.13:9944' create_category //Alice 'my cate'
	program.command('create_category <signer> <metadata>').action(async (signer, metadata) => {
		await create_category(program.opts().ws, keyring, signer, metadata);
	});
	// node nft-apis.mjs --ws 'ws://81.70.132.13:9944' show_category
	program.command('show_category').action(async () => {
		await show_category(program.opts().ws);
	});
	await program.parseAsync(process.argv);
}

async function show_category(ws) {
	await initApi(ws);
	let cateCount = 0;
	const callCategories = await Global_Api.query.nftmartConf.categories.entries();
	for (let category of callCategories) {
		let key = category[0];
		const data = category[1].unwrap();
		const len = key.length;
		key = key.buffer.slice(len - 8, len);
		const cateId = u32sToU64(new Uint32Array(key));
		console.log(cateId.toString(), data.toHuman());
		cateCount++;
	}
	console.log(`cateCount is ${cateCount}.`);
}

async function create_category(ws, keyring, signer, metadata) {
	await initApi(ws);
	signer = keyring.addFromUri(signer);
	const call = Global_Api.tx.sudo.sudo(Global_Api.tx.nftmartConf.createCategory(metadata));
	const feeInfo = await call.paymentInfo(signer);
	console.log("The fee of the call: %s.", feeInfo.partialFee / unit);
	let [a, b] = waitTx(Global_ModuleMetadata);
	await call.signAndSend(signer, a);
	await b();
}

async function destroy_class(ws, keyring, signer, classID) {
	await initApi(ws);
	await query_class_by(ws, keyring, signer);
	const sk = keyring.addFromUri(signer);
	let classInfo = await Global_Api.query.ormlNft.classes(classID);
	if (classInfo.isSome) {
		classInfo = classInfo.unwrap();
		const call = Global_Api.tx.proxy.proxy(classInfo.owner, null, Global_Api.tx.nftmart.destroyClass(classID, sk.address));
		const feeInfo = await call.paymentInfo(sk);
		console.log("The fee of the call: %s.", feeInfo.partialFee / unit);
		let [a, b] = waitTx(Global_ModuleMetadata);
		await call.signAndSend(sk, a);
		await b();
	}
	await query_class_by(ws, keyring, signer);
}

async function burn_nft(ws, keyring, signer, classID, tokenID, quantity) {
	await initApi(ws);
	await query_nft_by(ws, keyring, signer);
	const sk = keyring.addFromUri(signer);

	const call = Global_Api.tx.nftmart.burn(classID, tokenID, quantity);
	const feeInfo = await call.paymentInfo(sk);
	console.log("The fee of the call: %s.", feeInfo.partialFee / unit);
	let [a, b] = waitTx(Global_ModuleMetadata);
	await call.signAndSend(sk, a);
	await b();

	await query_nft_by(ws, keyring, signer);
}

async function transfer_nfts(ws, tokens, keyring, from_raw, to) {
	await initApi(ws);
	const from = keyring.addFromUri(from_raw);

	const call = Global_Api.tx.nftmart.transfer(ensureAddress(keyring, to), tokens);
	const feeInfo = await call.paymentInfo(from);
	console.log("The fee of the call: %s NMT.", feeInfo.partialFee / unit);

	let [a, b] = waitTx(Global_ModuleMetadata);
	await call.signAndSend(from, a);
	await b();

	console.log("from %s", from_raw);
	await query_nft_by(ws, keyring, from_raw);
	console.log("to %s", to);
	await query_nft_by(ws, keyring, to);
}

async function query_class_by(ws, keyring, account) {
	await initApi(ws);
	const address = ensureAddress(keyring, account);
	const allClasses = await Global_Api.query.ormlNft.classes.entries();
	for (const c of allClasses) {
		let key = c[0];
		const len = key.length;
		key = key.buffer.slice(len - 4, len);
		const classID = new Uint32Array(key)[0];
		let clazz = c[1].toJSON();
		clazz.metadata = hexToUtf8(clazz.metadata.slice(2));
		try{ clazz.metadata = JSON.parse(clazz.metadata); } catch(_e) {}
		clazz.classID = classID;
		clazz.adminList = await Global_Api.query.proxy.proxies(clazz.owner); // (Vec<ProxyDefinition>,BalanceOf)
		for (const a of clazz.adminList[0]) {
			if (a.delegate.toString() === address) {
				console.log("classInfo: %s", JSON.stringify(clazz));
			}
		}
	}
}

async function query_nft_by(ws, keyring, account) {
	await initApi(ws);
	const nfts = await Global_Api.query.ormlNft.tokensByOwner.entries(ensureAddress(keyring, account));
	for (let clzToken of nfts) {
		const accountToken = clzToken[1];
		clzToken = clzToken[0];
		const len = clzToken.length;

		const classID = new Uint32Array(clzToken.slice(len - 4 - 8, len - 8))[0];
		const tokenID = u32sToU64(new Uint32Array(clzToken.slice(len - 8, len)));

		let nft = await Global_Api.query.ormlNft.tokens(classID, tokenID);
		if (nft.isSome) {
			nft = nft.unwrap();
			console.log(`classID ${classID} tokenID ${tokenID} quantity ${accountToken} tokenInfo ${nft.toString()}`);
		}
	}
}

async function display_nft(classID) {
	let tokenCount = 0;
	let classInfo = await Global_Api.query.ormlNft.classes(classID);
	if (classInfo.isSome) {
		const nextTokenId = await Global_Api.query.ormlNft.nextTokenId(classID);
		console.log(`nextTokenId in classId ${classID} is ${nextTokenId}.`);
		classInfo = classInfo.unwrap();
		const accountInfo = await Global_Api.query.system.account(classInfo.owner);
		console.log("classInfo: %s", classInfo.toString());
		console.log("classOwner: %s", accountInfo.toString());
		for (let i = 0; i < nextTokenId; i++) {
			let nft = await Global_Api.query.ormlNft.tokens(classID, i);
			if (nft.isSome) {
				nft = nft.unwrap();
				nft = nft.toJSON();
				nft.metadata = hexToUtf8(nft.metadata.slice(2));
				try{ nft.metadata = JSON.parse(nft.metadata); } catch(_e) {}
				console.log("classId %s, tokenId %s, tokenInfo %s", classID, i, JSON.stringify(nft));
				tokenCount++;
			}
		}
	}
	console.log(`The token count of class ${classID} is ${tokenCount}.`);
}

async function show_nft(ws, classID) {
	await initApi(ws);
	if (classID === undefined) { // find all nfts
		const allClasses = await Global_Api.query.ormlNft.classes.entries();
		for (const c of allClasses) {
			let key = c[0];
			const len = key.length;
			key = key.buffer.slice(len - 4, len);
			const classID = new Uint32Array(key)[0];
			await display_nft(classID);
		}
	} else {
		await display_nft(classID);
	}
}

async function mint_nft(ws, keyring, admin, classID, quantity) {
	await initApi(ws);
	admin = keyring.addFromUri(admin);
	const classInfo = await Global_Api.query.ormlNft.classes(classID);
	if (classInfo.isSome) {
		const ownerOfClass = classInfo.unwrap().owner.toString();
		const nftMetadata = 'demo nft metadata';
		const balancesNeeded = await nftDeposit(nftMetadata);
		if (balancesNeeded === null) {
			return;
		}
		const needToChargeRoyalty = null; // follow the config in class.
		// const needToChargeRoyalty = true;
		// const needToChargeRoyalty = false;
		const txs = [
			// make sure `ownerOfClass` has sufficient balances to mint nft.
			Global_Api.tx.balances.transfer(ownerOfClass, balancesNeeded),
			// mint some nfts and transfer to admin.address.
			Global_Api.tx.proxy.proxy(ownerOfClass, null, Global_Api.tx.nftmart.mint(admin.address, classID, nftMetadata, quantity, needToChargeRoyalty)),
		];
		const batchExtrinsic = Global_Api.tx.utility.batchAll(txs);
		const feeInfo = await batchExtrinsic.paymentInfo(admin);
		console.log("fee of batchExtrinsic: %s NMT", feeInfo.partialFee / unit);

		let [a, b] = waitTx(Global_ModuleMetadata);
		await batchExtrinsic.signAndSend(admin, a);
		await b();
	}
}

async function add_class_admin(ws, keyring, admin, classId, newAdmin) {
	await initApi(ws);
	admin = keyring.addFromUri(admin);
	newAdmin = ensureAddress(keyring, newAdmin);
	let classInfo = await Global_Api.query.ormlNft.classes(classId);
	if(classInfo.isSome) {
		classInfo = classInfo.unwrap();
		const ownerOfClass = classInfo.owner;
		console.log(ownerOfClass.toString());
		const balancesNeeded = await proxyDeposit(1);
		if (balancesNeeded === null) {
			return;
		}
		console.log("adding a class admin needs to reserve %s NMT", balancesNeeded / unit);
		const txs = [
			// make sure `ownerOfClass` has sufficient balances.
			Global_Api.tx.balances.transfer(ownerOfClass, balancesNeeded),
			// Add `newAdmin` as a new admin.
			Global_Api.tx.proxy.proxy(ownerOfClass, null, Global_Api.tx.proxy.addProxy(newAdmin, 'Any', 0)),
			// Global_Api.tx.proxy.proxy(ownerOfClass, null, Global_Api.tx.proxy.removeProxy(newAdmin, 'Any', 0)), to remove an admin
		];
		const batchExtrinsic = Global_Api.tx.utility.batchAll(txs);
		const feeInfo = await batchExtrinsic.paymentInfo(admin);
		console.log("fee of batchExtrinsic: %s NMT", feeInfo.partialFee / unit);

		let [a, b] = waitTx(Global_ModuleMetadata);
		await batchExtrinsic.signAndSend(admin, a);
		await b();
	}
}

async function show_class(ws) {
	await initApi(ws);
	let classCount = 0;
	const allClasses = await Global_Api.query.ormlNft.classes.entries();
	let all = [];
	for (const c of allClasses) {
		let key = c[0];
		const len = key.length;
		key = key.buffer.slice(len - 4, len);
		const classID = new Uint32Array(key)[0];
		let clazz = c[1].toJSON();
		clazz.metadata = hexToUtf8(clazz.metadata.slice(2));
		try{ clazz.metadata = JSON.parse(clazz.metadata); } catch(_e) {}
		clazz.classID = classID;
		clazz.adminList = await Global_Api.query.proxy.proxies(clazz.owner);
		all.push(JSON.stringify(clazz));
		classCount++;
	}
	console.log("%s", all);
	console.log("class count: %s", classCount);
	console.log("nextClassId: %s", await Global_Api.query.ormlNft.nextClassId());
}

async function add_whitelist(ws, keyring, sudo, account) {
	// usage: node nft-apis.mjs add-whitelist //Alice 63dHdZZMdgFeHs544yboqnVvrnAaTRdPWPC1u2aZjpC5HTqx
	await initApi(ws);
	sudo = keyring.addFromUri(sudo);
	account = ensureAddress(keyring, account);
	// const call = Global_Api.tx.sudo.sudo(Global_Api.tx.config.removeWhitelist(account.address)); to remove account from whitelist
	const call = Global_Api.tx.sudo.sudo(Global_Api.tx.nftmartConf.addWhitelist(account));
	const feeInfo = await call.paymentInfo(sudo.address);
	console.log("The fee of the call: %s.", feeInfo.partialFee / unit);
	let [a, b] = waitTx(Global_ModuleMetadata);
	await call.signAndSend(sudo, a);
	await b();
}

async function show_whitelist(ws, keyring) {
	await initApi(ws);
	const all = await Global_Api.query.nftmartConf.accountWhitelist.entries();
	for (const account of all) {
		let key = account[0];
		const len = key.length;
		key = key.buffer.slice(len - 32, len);
		const addr = keyring.encodeAddress(new Uint8Array(key));
		console.log("%s", addr);
	}
}

async function create_class(ws, keyring, signer) {
	await initApi(ws);
	signer = keyring.addFromUri(signer);

	const name = 'demo class name';
	const description = 'demo class description';
	const metadata = 'demo class metadata';

	const deposit = await classDeposit(metadata, name, description);
	console.log("create class deposit %s", deposit);

	// 	Transferable = 0b00000001,
	// 	Burnable = 0b00000010,
	// 	RoyaltiesChargeable = 0b00000100,
	let [a, b] = waitTx(Global_ModuleMetadata);
	await Global_Api.tx.nftmart.createClass(metadata, name, description, 1 | 2 | 4).signAndSend(signer, a);
	await b();
}

main().then(r => {
	process.exit();
}).catch(err => {
	console.log(err);
	process.exit();
});
