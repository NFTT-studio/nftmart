const { ApiPromise, WsProvider } = require('@polkadot/api');

const convert = (from, to) => str => Buffer.from(str, from).toString(to)
const utf8ToHex = convert('utf8', 'hex')
const hexToUtf8 = convert('hex', 'utf8')

function sleep(milliseconds) {
	return new Promise(resolve => setTimeout(resolve, milliseconds))
}

function waitTx(moduleMetadata) {
	let signal = false;
	return [
		({ events = [], status }) => {
			// console.log(JSON.stringify(status));
			// if (status.isFinalized) { // 交易敲定
			if (status.isInBlock) { // 交易入块
				// console.log('%s BlockHash(%s)', status.type, status.asFinalized.toHex());
				console.log('%s BlockHash(%s)', status.type, status.asInBlock.toHex());
				events.forEach(({ phase, event: { data, method, section } }) => {
					if ("system.ExtrinsicFailed" === section + '.' + method) { // 错误
						for (let d of data) {
							if (d.isModule) {
								let mErr = d.asModule;
								let module = moduleMetadata[mErr.index];
								console.log("error: %s.%s", module.name, module.errors[mErr.error].name);
							}
						}
					} else if ("system.ExtrinsicSuccess" === section + '.' + method) {
						// ignore
					} else { // 事件
						console.log("event: " + phase.toString() + ' ' + section + '.' + method + ' ' + data.toString());
					}
				});
				signal = true;
			}
		},
		async function () {
			for (; ;) {
				await sleep(100);
				if (signal) break;
			}
		}
	];
}

async function getApi(dest = 'ws://8.136.111.191:9944') {
	const provider = new WsProvider(dest);

	const types = {

	};

	const api = await ApiPromise.create({ provider, types });
	const [chain, nodeName, nodeVersion] = await Promise.all([
		api.rpc.system.chain(),
		api.rpc.system.name(),
		api.rpc.system.version()
	]);
	console.log(`You are connected to chain ${chain} using ${nodeName} v${nodeVersion}`);
	return api;
}

function secondsToString(seconds) {
	let numyears = Math.floor(seconds / 31536000);
	let numdays = Math.floor((seconds % 31536000) / 86400);
	let numhours = Math.floor(((seconds % 31536000) % 86400) / 3600);
	let numminutes = Math.floor((((seconds % 31536000) % 86400) % 3600) / 60);
	let numseconds = (((seconds % 31536000) % 86400) % 3600) % 60;
	return numyears + " years " + numdays + " days " + numhours + " hours " + numminutes + " minutes " + Math.round(numseconds) + " seconds";
}

module.exports = {
	sleep,
	getApi,
	waitTx,
	utf8ToHex,
	secondsToString,
	hexToUtf8,
	async getModules(api) {
		let metadata = await api.rpc.state.getMetadata();
		metadata = metadata.asLatest.modules;
		// 建立索引
		metadata.index = {};
		for (const a of metadata) {
			metadata.index[a.index] = a;
		}
		return metadata;
	},
	showStorage(s, verbose) {
		console.log("********** storage ***************")
		// noinspection JSUnresolvedVariable
		if (!s.isNone) {
			let storage = s.unwrap();
			console.log("prefix in key-value databases: [%s]", storage.prefix);
			for (let s of storage.items) {
				// noinspection JSUnresolvedVariable
				console.log("%s: modifier[%s] %s", s.name, s.modifier, s.documentation[0]);
				if (verbose) console.log(s.toHuman());
			}
		}
	},
	showCalls(s, verbose) {
		console.log("********** calls ***************")
		// noinspection JSUnresolvedVariable
		if (!s.isNone) {
			let calls = s.unwrap();
			for (let s of calls) {
				// noinspection JSUnresolvedVariable
				console.log("%s: %s", s.name, s.documentation[0]);
				if (verbose) console.log(s.toHuman());
			}
		}
	},
	showErrors(errors, verbose) {
		console.log("********** errors ***************")
		// noinspection JSUnresolvedVariable
		for (let e of errors) {
			// noinspection JSUnresolvedVariable
			console.log("%s: %s", e.name, e.documentation[0]);
			if (verbose) console.log(e.toHuman());
		}
	},
	showEvents(e, verbose) {
		console.log("********** events ***************")
		// noinspection JSUnresolvedVariable
		if (!e.isNone) {
			let events = e.unwrap();
			for (let e of events) {
				// noinspection JSUnresolvedVariable
				console.log("%s: %s", e.name, e.documentation[0]);
				if (verbose) console.log(e.toHuman());
			}
		}
	},
	showConstants(constants) {
		console.log("********** constants ***************")
		for (let c of constants) {
			// noinspection JSUnresolvedVariable
			console.log("%s %s = %s", c.type, c.name, c.documentation);
		}
	},
	findModule(name, moduleMetadata) {
		for (let module of moduleMetadata) {
			// console.log(module.name.toHuman());
			if (name === module.name.toHuman()) {
				return module;
			}
		}
		return {};
	},
	findConstantFrom(name, module) {
		for (let c of module['constants']) {
			// console.log(module.name.toHuman());
			if (name === c.name.toHuman()) {
				return c;
			}
		}
		return {};
	},
	reverseEndian(x) {
		let buf = Buffer.allocUnsafe(4)
		buf.writeUIntLE(x, 0, 4)
		return buf.readUIntBE(0, 4)
	},
	async getEventsByNumber(api, num) {
		const hash = await api.rpc.chain.getBlockHash(num);
		const events = await api.query.system.events.at(hash);
		// noinspection JSUnresolvedFunction
		return [hash.toHex(), events];
	},
	async getExtrinsicByNumber(api, num) {
		const hash = await api.rpc.chain.getBlockHash(num);
		return api.rpc.chain.getBlock(hash);
		// block.block.extrinsics.forEach((ex, index) => {
		//     console.log(index, ex.method);
		// });
	},
}
