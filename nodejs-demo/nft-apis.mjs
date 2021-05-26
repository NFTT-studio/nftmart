import {getApi, getModules, waitTx, hexToUtf8, unit} from "./utils.mjs";
import {Keyring} from "@polkadot/api";
import {bnToBn} from "@polkadot/util";
import {Command} from "commander";

async function main() {
	const ss58Format = 50;
	const keyring = new Keyring({type: 'sr25519', ss58Format});
	const program = new Command();
	program.option('--ws <addr>', 'node ws addr', 'ws://192.168.0.2:9944');

	program.command('create_class <signer>').action(async (signer) => {
		await create_class(program.opts().ws, keyring, signer);
	});
	program.command('show_whitelist').action(async () => {
		await show_whitelist(program.opts().ws, keyring);
	});
	program.command('add_whitelist <sudo> <account>').action(async (sudo, account) => {
		await add_whitelist(program.opts().ws, keyring, sudo, account);
	});

	await program.parseAsync(process.argv);
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
