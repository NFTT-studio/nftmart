// This file is part of Substrate.

// Copyright (C) 2018-2021 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

//! Substrate chain configurations.

use sc_chain_spec::ChainSpecExtension;
use sp_core::{Pair, Public, crypto::UncheckedInto, sr25519};
use serde::{Serialize, Deserialize};
use node_runtime::{
	AuthorityDiscoveryConfig, BabeConfig, BalancesConfig, ContractsConfig, CouncilConfig,
	DemocracyConfig,GrandpaConfig, ImOnlineConfig, SessionConfig, SessionKeys, StakerStatus,
	StakingConfig, ElectionsConfig, IndicesConfig, SocietyConfig, SudoConfig, SystemConfig,
	TechnicalCommitteeConfig, wasm_binary_unwrap, TokensConfig, OrmlNFTConfig,
};
use node_runtime::Block;
use node_runtime::constants::currency::*;
use sc_service::ChainType;
use hex_literal::hex;
use sc_telemetry::TelemetryEndpoints;
use grandpa_primitives::{AuthorityId as GrandpaId};
use sp_consensus_babe::{AuthorityId as BabeId};
use pallet_im_online::sr25519::{AuthorityId as ImOnlineId};
use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
use sp_runtime::{Perbill, traits::{Verify, IdentifyAccount}};

pub use node_primitives::{AccountId, Balance, Signature};
pub use node_runtime::GenesisConfig;

type AccountPublic = <Signature as Verify>::Signer;

const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Node `ChainSpec` extensions.
///
/// Additional parameters for some Substrate core modules,
/// customizable from the chain spec.
#[derive(Default, Clone, Serialize, Deserialize, ChainSpecExtension)]
#[serde(rename_all = "camelCase")]
pub struct Extensions {
	/// Block numbers with known hashes.
	pub fork_blocks: sc_client_api::ForkBlocks<Block>,
	/// Known bad block hashes.
	pub bad_blocks: sc_client_api::BadBlocks<Block>,
}

/// Specialized `ChainSpec`.
pub type ChainSpec = sc_service::GenericChainSpec<
	GenesisConfig,
	Extensions,
>;
/// Flaming Fir testnet generator
pub fn flaming_fir_config() -> Result<ChainSpec, String> {
	ChainSpec::from_json_bytes(&include_bytes!("../res/flaming-fir.json")[..])
}

fn session_keys(
	grandpa: GrandpaId,
	babe: BabeId,
	im_online: ImOnlineId,
	authority_discovery: AuthorityDiscoveryId,
) -> SessionKeys {
	SessionKeys { grandpa, babe, im_online, authority_discovery }
}

fn staging_testnet_config_genesis() -> GenesisConfig {
	/*
#root: {"accountId":"0x04c9ad9d268df4cf71e4ca98d3d4ba84f1e17c6eeee190180dd89c1ada821f52","publicKey":"0x04c9ad9d268df4cf71e4ca98d3d4ba84f1e17c6eeee190180dd89c1ada821f52","secretPhrase":"trophy brush claw east grid grief pact brain common vehicle rare carpet","secretSeed":"0x02d709290f59204c7f6e3bd047f39d01a403b13844a27b996ca0bb89ad303030","ss58Address":"5zUG2YnhDNZ9a8fyyNvU1ebf7bMt2yP7dxhhG5jvhGecNatw"}
#stash1: {"accountId":"0xc0fa20c97586101d479a70a3881f6c9012b55ef2f3cd07ebb0b8f6b8cb1fde3f","publicKey":"0xc0fa20c97586101d479a70a3881f6c9012b55ef2f3cd07ebb0b8f6b8cb1fde3f","secretKeyUri":"trophy brush claw east grid grief pact brain common vehicle rare carpet//nftmart/stash/1","secretSeed":"n/a","ss58Address":"64j1RTQ5yWNDE4sCRYZxWzJafwS8VMKDnj1yD4pwtyqLDyBu"}
#controller1: {"accountId":"0x1a92ecf4f212293c8588af4d6bfb351624a145594840c68096c5d9d95a99f87b","publicKey":"0x1a92ecf4f212293c8588af4d6bfb351624a145594840c68096c5d9d95a99f87b","secretKeyUri":"trophy brush claw east grid grief pact brain common vehicle rare carpet//nftmart/controller/1","secretSeed":"n/a","ss58Address":"5zxppHV1d6W32Y5tekm8tJyKUW2uJZVBGycQ5K3Vg6ZbzatU"}
#stash2: {"accountId":"0xb4eb5293dc20e64dec2de27a284961c143b7a3bfdaeee85c5fc48f797e425d73","publicKey":"0xb4eb5293dc20e64dec2de27a284961c143b7a3bfdaeee85c5fc48f797e425d73","secretKeyUri":"trophy brush claw east grid grief pact brain common vehicle rare carpet//nftmart/stash/2","secretSeed":"n/a","ss58Address":"64TCT2zwe86oT4EFLY4FBsVCYmWccWAbswVLVLnoNjbBsG5X"}
#controller2: {"accountId":"0x56be3faef9ec04c2742e7570deeb7400c3086fee44d624fcf401f99a1dcfc746","publicKey":"0x56be3faef9ec04c2742e7570deeb7400c3086fee44d624fcf401f99a1dcfc746","secretKeyUri":"trophy brush claw east grid grief pact brain common vehicle rare carpet//nftmart/controller/2","secretSeed":"n/a","ss58Address":"62KiZN5jayad35D6dHRp3GsFsQxK2r5mYFMPWcBzAJPo3UEp"}
#stash3: {"accountId":"0x0af3e045a8b2ac53106aeada5859063b7b337ee7b0bbe7ca675fe1412f909d62","publicKey":"0x0af3e045a8b2ac53106aeada5859063b7b337ee7b0bbe7ca675fe1412f909d62","secretKeyUri":"trophy brush claw east grid grief pact brain common vehicle rare carpet//nftmart/stash/3","secretSeed":"n/a","ss58Address":"5zcLrGLxWfu4FHz2116Cbhk78PS8XhUNiMapPG96oHPKGTL2"}
#controller3: {"accountId":"0x383e4059667d36df8b122377a1fd00442a50ba18585352affa4d3625b28aa858","publicKey":"0x383e4059667d36df8b122377a1fd00442a50ba18585352affa4d3625b28aa858","secretKeyUri":"trophy brush claw east grid grief pact brain common vehicle rare carpet//nftmart/controller/3","secretSeed":"n/a","ss58Address":"61dj6hcc19UytcnyzrsopHwpSRc3rsZD87fdf2LmpaGHsuWF"}
#stash4: {"accountId":"0x92e2c73f83a9bd90271962e689174dfc453b4671d629b13fce4bda407633d628","publicKey":"0x92e2c73f83a9bd90271962e689174dfc453b4671d629b13fce4bda407633d628","secretKeyUri":"trophy brush claw east grid grief pact brain common vehicle rare carpet//nftmart/stash/4","secretSeed":"n/a","ss58Address":"63gaHPimxJ9FVvjKiPXwZ8odhJzjYf1DsjhTye3uhq6KQ8ht"}
#controller4: {"accountId":"0x688bbc0e7c4f01bfe5c65e12597fac23ef5578a765a4120fe70b9a616d2d5b5b","publicKey":"0x688bbc0e7c4f01bfe5c65e12597fac23ef5578a765a4120fe70b9a616d2d5b5b","secretKeyUri":"trophy brush claw east grid grief pact brain common vehicle rare carpet//nftmart/controller/4","secretSeed":"n/a","ss58Address":"62j4R1whrXuHCVQgY3WvDV6dpsf6Zv6cMkC36tiUcvRFWhvE"}
local ip_of_node1=11.22.33.41
local ip_of_node2=11.22.33.42
local ip_of_node3=11.22.33.43
local ip_of_node4=11.22.33.44
#grandpa1: {"accountId":"0xa1f84dd2eb7b959aca9c80bedae5e01a66aad76105f92b651df17b7638a5b772","publicKey":"0xa1f84dd2eb7b959aca9c80bedae5e01a66aad76105f92b651df17b7638a5b772","secretKeyUri":"trophy brush claw east grid grief pact brain common vehicle rare carpet//nftmart//grandpa//1","secretSeed":"0x468c7fe28d9afc194be5931ba9b21015a216c295dc73b678a4c1157637253a4e","ss58Address":"642MPvxmGF75GNqjDcBV2LawhHPzfiDKS2jeuDKQR2XH3sQ6"}
#babe1: {"accountId":"0x10d9d0032df78215fda4b8abba286ba87f8beff50a384024b04144d69fe9625c","publicKey":"0x10d9d0032df78215fda4b8abba286ba87f8beff50a384024b04144d69fe9625c","secretKeyUri":"trophy brush claw east grid grief pact brain common vehicle rare carpet//nftmart//babe//1","secretSeed":"0xfa8d14db7d9864f2e939be79f800d2eabb94768a50a2f114a5ccc27f010324ce","ss58Address":"5zk5Ps9HaP82z5HBkrphwgsPfoKEkfHqCDbvq3spgtfMTYxv"}
ssh -o IdentitiesOnly=yes root@$ip_of_node1 <<'EOF'
curl --header "Content-Type:application/json;charset=utf-8" --request POST --data '{"jsonrpc":"2.0", "id":1, "method":"author_insertKey", "params": ["gran", "trophy brush claw east grid grief pact brain common vehicle rare carpet//nftmart//grandpa//1", "0xa1f84dd2eb7b959aca9c80bedae5e01a66aad76105f92b651df17b7638a5b772"]}' http://localhost:9933
curl --header "Content-Type:application/json;charset=utf-8" --request POST --data '{"jsonrpc":"2.0", "id":1, "method":"author_insertKey", "params": ["babe", "trophy brush claw east grid grief pact brain common vehicle rare carpet//nftmart//babe//1", "0x10d9d0032df78215fda4b8abba286ba87f8beff50a384024b04144d69fe9625c"]}' http://localhost:9933
curl --header "Content-Type:application/json;charset=utf-8" --request POST --data '{"jsonrpc":"2.0", "id":1, "method":"author_insertKey", "params": ["imon", "trophy brush claw east grid grief pact brain common vehicle rare carpet//nftmart//babe//1", "0x10d9d0032df78215fda4b8abba286ba87f8beff50a384024b04144d69fe9625c"]}' http://localhost:9933
curl --header "Content-Type:application/json;charset=utf-8" --request POST --data '{"jsonrpc":"2.0", "id":1, "method":"author_insertKey", "params": ["audi", "trophy brush claw east grid grief pact brain common vehicle rare carpet//nftmart//babe//1", "0x10d9d0032df78215fda4b8abba286ba87f8beff50a384024b04144d69fe9625c"]}' http://localhost:9933
EOF
#grandpa2: {"accountId":"0x79b613d7f69c651548a6e3c55de81341597885c2cc00ec2f3c06a1ec22fce802","publicKey":"0x79b613d7f69c651548a6e3c55de81341597885c2cc00ec2f3c06a1ec22fce802","secretKeyUri":"trophy brush claw east grid grief pact brain common vehicle rare carpet//nftmart//grandpa//2","secretSeed":"0xd0e83caea28c17dc5dc2274bfb30bfbc3f37f230a15324b498da1f09c487aa49","ss58Address":"637ZonMtzvyUUfQX2Jj1RWYCrFfFUBbYTTSQQfzDLHDS2j86"}
#babe2: {"accountId":"0xfa7c03a0da328ee334901fcc286907a4aa0f8ba17680f793e8b9d922f8caa005","publicKey":"0xfa7c03a0da328ee334901fcc286907a4aa0f8ba17680f793e8b9d922f8caa005","secretKeyUri":"trophy brush claw east grid grief pact brain common vehicle rare carpet//nftmart//babe//2","secretSeed":"0xa0d41fed16c57d3ba684bca25f0a2927378bab39353f38b383cb2010d991d77b","ss58Address":"662QjeYigW8dMgin5ts36vjmL8jbHtDFkncXACZ3Ra8Z9jJs"}
ssh -o IdentitiesOnly=yes root@$ip_of_node2 <<'EOF'
curl --header "Content-Type:application/json;charset=utf-8" --request POST --data '{"jsonrpc":"2.0", "id":1, "method":"author_insertKey", "params": ["gran", "trophy brush claw east grid grief pact brain common vehicle rare carpet//nftmart//grandpa//2", "0x79b613d7f69c651548a6e3c55de81341597885c2cc00ec2f3c06a1ec22fce802"]}' http://localhost:9933
curl --header "Content-Type:application/json;charset=utf-8" --request POST --data '{"jsonrpc":"2.0", "id":1, "method":"author_insertKey", "params": ["babe", "trophy brush claw east grid grief pact brain common vehicle rare carpet//nftmart//babe//2", "0xfa7c03a0da328ee334901fcc286907a4aa0f8ba17680f793e8b9d922f8caa005"]}' http://localhost:9933
curl --header "Content-Type:application/json;charset=utf-8" --request POST --data '{"jsonrpc":"2.0", "id":1, "method":"author_insertKey", "params": ["imon", "trophy brush claw east grid grief pact brain common vehicle rare carpet//nftmart//babe//2", "0xfa7c03a0da328ee334901fcc286907a4aa0f8ba17680f793e8b9d922f8caa005"]}' http://localhost:9933
curl --header "Content-Type:application/json;charset=utf-8" --request POST --data '{"jsonrpc":"2.0", "id":1, "method":"author_insertKey", "params": ["audi", "trophy brush claw east grid grief pact brain common vehicle rare carpet//nftmart//babe//2", "0xfa7c03a0da328ee334901fcc286907a4aa0f8ba17680f793e8b9d922f8caa005"]}' http://localhost:9933
EOF
#grandpa3: {"accountId":"0xdadc465a4f4619289f161acf3728be6e414b12189cc530c5429421dd609a6962","publicKey":"0xdadc465a4f4619289f161acf3728be6e414b12189cc530c5429421dd609a6962","secretKeyUri":"trophy brush claw east grid grief pact brain common vehicle rare carpet//nftmart//grandpa//3","secretSeed":"0x59859e565d8711c4b8a4bca0ebe1345e799c9f9e061f4e78b6b0512facf0491f","ss58Address":"65JwoJj3x9hJJJRcKHzsEbzUuY8BTHBWx1M9mY2jsCQ7HeYh"}
#babe3: {"accountId":"0x1c6051d240e885f89f8829c1fa7d5f4c7ee6b7864828633cea7c008c653e944a","publicKey":"0x1c6051d240e885f89f8829c1fa7d5f4c7ee6b7864828633cea7c008c653e944a","secretKeyUri":"trophy brush claw east grid grief pact brain common vehicle rare carpet//nftmart//babe//3","secretSeed":"0xa6daadac3b17c63319e784da12f0268c23291cc404ec9e2cf6948ce1c7c0a547","ss58Address":"611BsvENC8Zet78pT2Yyqb2Lr1SwTcoPbpeynnypNoRJxDPn"}
ssh -o IdentitiesOnly=yes root@$ip_of_node3 <<'EOF'
curl --header "Content-Type:application/json;charset=utf-8" --request POST --data '{"jsonrpc":"2.0", "id":1, "method":"author_insertKey", "params": ["gran", "trophy brush claw east grid grief pact brain common vehicle rare carpet//nftmart//grandpa//3", "0xdadc465a4f4619289f161acf3728be6e414b12189cc530c5429421dd609a6962"]}' http://localhost:9933
curl --header "Content-Type:application/json;charset=utf-8" --request POST --data '{"jsonrpc":"2.0", "id":1, "method":"author_insertKey", "params": ["babe", "trophy brush claw east grid grief pact brain common vehicle rare carpet//nftmart//babe//3", "0x1c6051d240e885f89f8829c1fa7d5f4c7ee6b7864828633cea7c008c653e944a"]}' http://localhost:9933
curl --header "Content-Type:application/json;charset=utf-8" --request POST --data '{"jsonrpc":"2.0", "id":1, "method":"author_insertKey", "params": ["imon", "trophy brush claw east grid grief pact brain common vehicle rare carpet//nftmart//babe//3", "0x1c6051d240e885f89f8829c1fa7d5f4c7ee6b7864828633cea7c008c653e944a"]}' http://localhost:9933
curl --header "Content-Type:application/json;charset=utf-8" --request POST --data '{"jsonrpc":"2.0", "id":1, "method":"author_insertKey", "params": ["audi", "trophy brush claw east grid grief pact brain common vehicle rare carpet//nftmart//babe//3", "0x1c6051d240e885f89f8829c1fa7d5f4c7ee6b7864828633cea7c008c653e944a"]}' http://localhost:9933
EOF
#grandpa4: {"accountId":"0xee7e1001addf9b060c83920c8e013f4063474dd32225a878051834c8f0d3cb60","publicKey":"0xee7e1001addf9b060c83920c8e013f4063474dd32225a878051834c8f0d3cb60","secretKeyUri":"trophy brush claw east grid grief pact brain common vehicle rare carpet//nftmart//grandpa//4","secretSeed":"0x3b11af913c74341aa1a01c2bb04f6ef27b707aa463f224fdef407e21fcaaa968","ss58Address":"65kgmcVRQ6HYRHx8Pycrs73yXy8ocjSmEuy7qSyTx17ZpDXt"}
#babe4: {"accountId":"0x2820c1a44a60beb0fed206d5c6853f567bf5e6d4b596953f6fe71400aa773807","publicKey":"0x2820c1a44a60beb0fed206d5c6853f567bf5e6d4b596953f6fe71400aa773807","secretKeyUri":"trophy brush claw east grid grief pact brain common vehicle rare carpet//nftmart//babe//4","secretSeed":"0x60fc0c71cf528202468633b5e82b436fd87280a6a72a60db3653a60cf982398f","ss58Address":"61Gba5UDx489DCE1CruypdvS33NVpv7wqBHxcSFuCYNqsN8f"}
ssh -o IdentitiesOnly=yes root@$ip_of_node4 <<'EOF'
curl --header "Content-Type:application/json;charset=utf-8" --request POST --data '{"jsonrpc":"2.0", "id":1, "method":"author_insertKey", "params": ["gran", "trophy brush claw east grid grief pact brain common vehicle rare carpet//nftmart//grandpa//4", "0xee7e1001addf9b060c83920c8e013f4063474dd32225a878051834c8f0d3cb60"]}' http://localhost:9933
curl --header "Content-Type:application/json;charset=utf-8" --request POST --data '{"jsonrpc":"2.0", "id":1, "method":"author_insertKey", "params": ["babe", "trophy brush claw east grid grief pact brain common vehicle rare carpet//nftmart//babe//4", "0x2820c1a44a60beb0fed206d5c6853f567bf5e6d4b596953f6fe71400aa773807"]}' http://localhost:9933
curl --header "Content-Type:application/json;charset=utf-8" --request POST --data '{"jsonrpc":"2.0", "id":1, "method":"author_insertKey", "params": ["imon", "trophy brush claw east grid grief pact brain common vehicle rare carpet//nftmart//babe//4", "0x2820c1a44a60beb0fed206d5c6853f567bf5e6d4b596953f6fe71400aa773807"]}' http://localhost:9933
curl --header "Content-Type:application/json;charset=utf-8" --request POST --data '{"jsonrpc":"2.0", "id":1, "method":"author_insertKey", "params": ["audi", "trophy brush claw east grid grief pact brain common vehicle rare carpet//nftmart//babe//4", "0x2820c1a44a60beb0fed206d5c6853f567bf5e6d4b596953f6fe71400aa773807"]}' http://localhost:9933
EOF
p2p node1:
12D3KooWPzJJesMBHMY7UrkD85RpzLphjmJ3QVoCuVR5nB4VthQ7
5accac8d4366c9d486e438ffd840c7a89eaf163eabdb74fefdfbb08ca9eabd63
p2p node2:
12D3KooWF4MDrHwB5D1k6d7dyj6xm6p4cuuaJmQk75CX3CQwTKh2
eefd9269e5aed3115fd7da0e024609ccb542e8966efada26426181563356a779
p2p node3:
12D3KooWC9hPJjizMhvEaro2W1u5vG2yds9wGXoR5Pua6GcBRqYU
26d1e75399b5546090d0af427537e0f971c2830972b092c3670d429b4ab4e10e
p2p node4:
12D3KooWE3Fzq3HHPNmVmMn2EXxLzDzjqXucqAvxagZrCwqCuxLU
f88110820f61d324e06f1961ba17e125b451030678ced732c4e48b7f0e217af3
*/
	let root_key: AccountId = hex!["04c9ad9d268df4cf71e4ca98d3d4ba84f1e17c6eeee190180dd89c1ada821f52"].into(); // 5zUG2YnhDNZ9a8fyyNvU1ebf7bMt2yP7dxhhG5jvhGecNatw
	let stash1: AccountId = hex!["c0fa20c97586101d479a70a3881f6c9012b55ef2f3cd07ebb0b8f6b8cb1fde3f"].into(); // 64j1RTQ5yWNDE4sCRYZxWzJafwS8VMKDnj1yD4pwtyqLDyBu
	let controller1: AccountId = hex!["1a92ecf4f212293c8588af4d6bfb351624a145594840c68096c5d9d95a99f87b"].into(); // 5zxppHV1d6W32Y5tekm8tJyKUW2uJZVBGycQ5K3Vg6ZbzatU
	let stash2: AccountId = hex!["b4eb5293dc20e64dec2de27a284961c143b7a3bfdaeee85c5fc48f797e425d73"].into(); // 64TCT2zwe86oT4EFLY4FBsVCYmWccWAbswVLVLnoNjbBsG5X
	let controller2: AccountId = hex!["56be3faef9ec04c2742e7570deeb7400c3086fee44d624fcf401f99a1dcfc746"].into(); // 62KiZN5jayad35D6dHRp3GsFsQxK2r5mYFMPWcBzAJPo3UEp
	let stash3: AccountId = hex!["0af3e045a8b2ac53106aeada5859063b7b337ee7b0bbe7ca675fe1412f909d62"].into(); // 5zcLrGLxWfu4FHz2116Cbhk78PS8XhUNiMapPG96oHPKGTL2
	let controller3: AccountId = hex!["383e4059667d36df8b122377a1fd00442a50ba18585352affa4d3625b28aa858"].into(); // 61dj6hcc19UytcnyzrsopHwpSRc3rsZD87fdf2LmpaGHsuWF
	let stash4: AccountId = hex!["92e2c73f83a9bd90271962e689174dfc453b4671d629b13fce4bda407633d628"].into(); // 63gaHPimxJ9FVvjKiPXwZ8odhJzjYf1DsjhTye3uhq6KQ8ht
	let controller4: AccountId = hex!["688bbc0e7c4f01bfe5c65e12597fac23ef5578a765a4120fe70b9a616d2d5b5b"].into(); // 62j4R1whrXuHCVQgY3WvDV6dpsf6Zv6cMkC36tiUcvRFWhvE
	let initial_authorities: Vec<(AccountId, AccountId, GrandpaId, BabeId, ImOnlineId, AuthorityDiscoveryId)> = vec![
		(stash1, controller1,
		 hex!["a1f84dd2eb7b959aca9c80bedae5e01a66aad76105f92b651df17b7638a5b772"].unchecked_into(), // 642MPvxmGF75GNqjDcBV2LawhHPzfiDKS2jeuDKQR2XH3sQ6
		 hex!["10d9d0032df78215fda4b8abba286ba87f8beff50a384024b04144d69fe9625c"].unchecked_into(), // 5zk5Ps9HaP82z5HBkrphwgsPfoKEkfHqCDbvq3spgtfMTYxv
		 hex!["10d9d0032df78215fda4b8abba286ba87f8beff50a384024b04144d69fe9625c"].unchecked_into(), // 5zk5Ps9HaP82z5HBkrphwgsPfoKEkfHqCDbvq3spgtfMTYxv
		 hex!["10d9d0032df78215fda4b8abba286ba87f8beff50a384024b04144d69fe9625c"].unchecked_into(),), // 5zk5Ps9HaP82z5HBkrphwgsPfoKEkfHqCDbvq3spgtfMTYxv
		(stash2, controller2,
		 hex!["79b613d7f69c651548a6e3c55de81341597885c2cc00ec2f3c06a1ec22fce802"].unchecked_into(), // 637ZonMtzvyUUfQX2Jj1RWYCrFfFUBbYTTSQQfzDLHDS2j86
		 hex!["fa7c03a0da328ee334901fcc286907a4aa0f8ba17680f793e8b9d922f8caa005"].unchecked_into(), // 662QjeYigW8dMgin5ts36vjmL8jbHtDFkncXACZ3Ra8Z9jJs
		 hex!["fa7c03a0da328ee334901fcc286907a4aa0f8ba17680f793e8b9d922f8caa005"].unchecked_into(), // 662QjeYigW8dMgin5ts36vjmL8jbHtDFkncXACZ3Ra8Z9jJs
		 hex!["fa7c03a0da328ee334901fcc286907a4aa0f8ba17680f793e8b9d922f8caa005"].unchecked_into(),), // 662QjeYigW8dMgin5ts36vjmL8jbHtDFkncXACZ3Ra8Z9jJs
		(stash3, controller3,
		 hex!["dadc465a4f4619289f161acf3728be6e414b12189cc530c5429421dd609a6962"].unchecked_into(), // 65JwoJj3x9hJJJRcKHzsEbzUuY8BTHBWx1M9mY2jsCQ7HeYh
		 hex!["1c6051d240e885f89f8829c1fa7d5f4c7ee6b7864828633cea7c008c653e944a"].unchecked_into(), // 611BsvENC8Zet78pT2Yyqb2Lr1SwTcoPbpeynnypNoRJxDPn
		 hex!["1c6051d240e885f89f8829c1fa7d5f4c7ee6b7864828633cea7c008c653e944a"].unchecked_into(), // 611BsvENC8Zet78pT2Yyqb2Lr1SwTcoPbpeynnypNoRJxDPn
		 hex!["1c6051d240e885f89f8829c1fa7d5f4c7ee6b7864828633cea7c008c653e944a"].unchecked_into(),), // 611BsvENC8Zet78pT2Yyqb2Lr1SwTcoPbpeynnypNoRJxDPn
		(stash4, controller4,
		 hex!["ee7e1001addf9b060c83920c8e013f4063474dd32225a878051834c8f0d3cb60"].unchecked_into(), // 65kgmcVRQ6HYRHx8Pycrs73yXy8ocjSmEuy7qSyTx17ZpDXt
		 hex!["2820c1a44a60beb0fed206d5c6853f567bf5e6d4b596953f6fe71400aa773807"].unchecked_into(), // 61Gba5UDx489DCE1CruypdvS33NVpv7wqBHxcSFuCYNqsN8f
		 hex!["2820c1a44a60beb0fed206d5c6853f567bf5e6d4b596953f6fe71400aa773807"].unchecked_into(), // 61Gba5UDx489DCE1CruypdvS33NVpv7wqBHxcSFuCYNqsN8f
		 hex!["2820c1a44a60beb0fed206d5c6853f567bf5e6d4b596953f6fe71400aa773807"].unchecked_into(),), // 61Gba5UDx489DCE1CruypdvS33NVpv7wqBHxcSFuCYNqsN8f
	];

	let endowed_accounts: Vec<AccountId> = vec![root_key.clone()];

	testnet_genesis(
		initial_authorities,
		root_key,
		Some(endowed_accounts),
		false,
	)
}

/// Staging testnet config.
pub fn staging_testnet_config() -> ChainSpec {
	let mut prop = sc_service::Properties::new();
	prop.insert("tokenDecimals".to_string(), 12.into());
	prop.insert("tokenSymbol".to_string(), "NMT".into()); // NFT Mart Token
	let boot_nodes = vec![

	];
	ChainSpec::from_genesis(
		"Nftmart Staging",
		"nftmart_staging",
		ChainType::Live,
		staging_testnet_config_genesis,
		boot_nodes,
		Some(TelemetryEndpoints::new(vec![(STAGING_TELEMETRY_URL.to_string(), 0)])
			.expect("Staging telemetry url is valid; qed")),
		Some("nftmart"),
		Some(prop),
		Default::default(),
	)
}

/// Helper function to generate a crypto pair from seed
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

/// Helper function to generate an account ID from seed
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>
{
	AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Helper function to generate stash, controller and session key from seed
pub fn authority_keys_from_seed(seed: &str) -> (
	AccountId,
	AccountId,
	GrandpaId,
	BabeId,
	ImOnlineId,
	AuthorityDiscoveryId,
) {
	(
		get_account_id_from_seed::<sr25519::Public>(&format!("{}//stash", seed)),
		get_account_id_from_seed::<sr25519::Public>(seed),
		get_from_seed::<GrandpaId>(seed),
		get_from_seed::<BabeId>(seed),
		get_from_seed::<ImOnlineId>(seed),
		get_from_seed::<AuthorityDiscoveryId>(seed),
	)
}

/// Helper function to create GenesisConfig for testing
pub fn testnet_genesis(
	initial_authorities: Vec<(
		AccountId,
		AccountId,
		GrandpaId,
		BabeId,
		ImOnlineId,
		AuthorityDiscoveryId,
	)>,
	root_key: AccountId,
	endowed_accounts: Option<Vec<AccountId>>,
	enable_println: bool,
) -> GenesisConfig {
	let mut endowed_accounts: Vec<AccountId> = endowed_accounts.unwrap_or_else(|| {
		vec![
			get_account_id_from_seed::<sr25519::Public>("Alice"),
			get_account_id_from_seed::<sr25519::Public>("Bob"),
			get_account_id_from_seed::<sr25519::Public>("Charlie"),
			get_account_id_from_seed::<sr25519::Public>("Dave"),
			get_account_id_from_seed::<sr25519::Public>("Eve"),
			get_account_id_from_seed::<sr25519::Public>("Ferdie"),
			get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
			get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
			get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
			get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
			get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
			get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
		]
	});
	initial_authorities.iter().for_each(|x|
		if !endowed_accounts.contains(&x.0) {
			endowed_accounts.push(x.0.clone())
		}
	);

	let num_endowed_accounts = endowed_accounts.len();

	const ENDOWMENT: Balance = 10_000_000 * DOLLARS;
	const STASH: Balance = ENDOWMENT / 1000;

	GenesisConfig {
		frame_system: SystemConfig {
			code: wasm_binary_unwrap().to_vec(),
			changes_trie_config: Default::default(),
		},
		pallet_balances: BalancesConfig {
			balances: endowed_accounts.iter().cloned()
				.map(|x| (x, ENDOWMENT))
				.collect()
		},
		pallet_indices: IndicesConfig {
			indices: vec![],
		},
		pallet_session: SessionConfig {
			keys: initial_authorities.iter().map(|x| {
				(x.0.clone(), x.0.clone(), session_keys(
					x.2.clone(),
					x.3.clone(),
					x.4.clone(),
					x.5.clone(),
				))
			}).collect::<Vec<_>>(),
		},
		pallet_staking: StakingConfig {
			validator_count: initial_authorities.len() as u32 * 2,
			minimum_validator_count: initial_authorities.len() as u32,
			stakers: initial_authorities.iter().map(|x| {
				(x.0.clone(), x.1.clone(), STASH, StakerStatus::Validator)
			}).collect(),
			invulnerables: initial_authorities.iter().map(|x| x.0.clone()).collect(),
			slash_reward_fraction: Perbill::from_percent(10),
			.. Default::default()
		},
		pallet_democracy: DemocracyConfig::default(),
		pallet_elections_phragmen: ElectionsConfig {
			members: endowed_accounts.iter()
						.take((num_endowed_accounts + 1) / 2)
						.cloned()
						.map(|member| (member, STASH))
						.collect(),
		},
		pallet_collective_Instance1: CouncilConfig::default(),
		pallet_collective_Instance2: TechnicalCommitteeConfig {
			members: endowed_accounts.iter()
						.take((num_endowed_accounts + 1) / 2)
						.cloned()
						.collect(),
			phantom: Default::default(),
		},
		pallet_contracts: ContractsConfig {
			// println should only be enabled on development chains
			current_schedule: pallet_contracts::Schedule::default()
				.enable_println(enable_println),
		},
		pallet_sudo: SudoConfig {
			key: root_key,
		},
		pallet_babe: BabeConfig {
			authorities: vec![],
			epoch_config: Some(node_runtime::BABE_GENESIS_EPOCH_CONFIG),
		},
		pallet_im_online: ImOnlineConfig {
			keys: vec![],
		},
		pallet_authority_discovery: AuthorityDiscoveryConfig {
			keys: vec![],
		},
		pallet_grandpa: GrandpaConfig {
			authorities: vec![],
		},
		pallet_membership_Instance1: Default::default(),
		pallet_treasury: Default::default(),
		pallet_society: SocietyConfig {
			members: endowed_accounts.iter()
						.take((num_endowed_accounts + 1) / 2)
						.cloned()
						.collect(),
			pot: 0,
			max_members: 999,
		},
		pallet_vesting: Default::default(),
		pallet_gilt: Default::default(),
		orml_tokens: TokensConfig {
			endowed_accounts: endowed_accounts.iter()
				.flat_map(|x|{
					vec![
						(x.clone(), 2, 100 * sp_core::constants_types::ACCURACY),
						(x.clone(), 3, 100 * sp_core::constants_types::ACCURACY),
						(x.clone(), 4, 100 * sp_core::constants_types::ACCURACY),
					]
				}).collect(),
		},
		orml_nft: OrmlNFTConfig { tokens: vec![] },
		nftmart_nft: Default::default(),
	}
}

fn development_config_genesis() -> GenesisConfig {
	testnet_genesis(
		vec![
			authority_keys_from_seed("Alice"),
		],
		get_account_id_from_seed::<sr25519::Public>("Alice"),
		None,
		true,
	)
}

/// Development config (single validator Alice)
pub fn development_config() -> ChainSpec {
	let mut prop = sc_service::Properties::new();
	prop.insert("tokenDecimals".to_string(), 12.into());
	prop.insert("tokenSymbol".to_string(), "NMT".into()); // NFT Mart Token
	ChainSpec::from_genesis(
		"Nftmart Testnet",
		"nftmart_testnet",
		ChainType::Development,
		development_config_genesis,
		vec![],
		None,
		Some("nftmart"),
		Some(prop),
		Default::default(),
	)
}

fn local_testnet_genesis() -> GenesisConfig {
	testnet_genesis(
		vec![
			authority_keys_from_seed("Alice"),
			authority_keys_from_seed("Bob"),
		],
		get_account_id_from_seed::<sr25519::Public>("Alice"),
		None,
		false,
	)
}

/// Local testnet config (multivalidator Alice + Bob)
pub fn local_testnet_config() -> ChainSpec {
	let mut prop = sc_service::Properties::new();
	prop.insert("tokenDecimals".to_string(), 12.into());
	prop.insert("tokenSymbol".to_string(), "NMT".into()); // NFT Mart Token
	ChainSpec::from_genesis(
		"Nftmart Testnet",
		"nftmart_testnet",
		ChainType::Local,
		local_testnet_genesis,
		vec![],
		Some(TelemetryEndpoints::new(vec![(STAGING_TELEMETRY_URL.to_string(), 0)])
			.expect("Local Testnet telemetry url is valid; qed")),
		Some("nftmart"),
		Some(prop),
		Default::default(),
	)
}

#[cfg(test)]
pub(crate) mod tests {
	use super::*;
	use crate::service::{new_full_base, new_light_base, NewFullBase};
	use sc_service_test;
	use sp_runtime::BuildStorage;

	fn local_testnet_genesis_instant_single() -> GenesisConfig {
		testnet_genesis(
			vec![
				authority_keys_from_seed("Alice"),
			],
			get_account_id_from_seed::<sr25519::Public>("Alice"),
			None,
			false,
		)
	}

	/// Local testnet config (single validator - Alice)
	pub fn integration_test_config_with_single_authority() -> ChainSpec {
		ChainSpec::from_genesis(
			"Integration Test",
			"test",
			ChainType::Development,
			local_testnet_genesis_instant_single,
			vec![],
			None,
			None,
			None,
			Default::default(),
		)
	}

	/// Local testnet config (multivalidator Alice + Bob)
	pub fn integration_test_config_with_two_authorities() -> ChainSpec {
		ChainSpec::from_genesis(
			"Integration Test",
			"test",
			ChainType::Development,
			local_testnet_genesis,
			vec![],
			None,
			None,
			None,
			Default::default(),
		)
	}

	#[test]
	#[ignore]
	fn test_connectivity() {
		sc_service_test::connectivity(
			integration_test_config_with_two_authorities(),
			|config| {
				let NewFullBase { task_manager, client, network, transaction_pool, .. }
					= new_full_base(config,|_, _| ())?;
				Ok(sc_service_test::TestNetComponents::new(task_manager, client, network, transaction_pool))
			},
			|config| {
				let (keep_alive, _, client, network, transaction_pool) = new_light_base(config)?;
				Ok(sc_service_test::TestNetComponents::new(keep_alive, client, network, transaction_pool))
			}
		);
	}

	#[test]
	fn test_create_development_chain_spec() {
		development_config().build_storage().unwrap();
	}

	#[test]
	fn test_create_local_testnet_chain_spec() {
		local_testnet_config().build_storage().unwrap();
	}

	#[test]
	fn test_staging_test_net_chain_spec() {
		staging_testnet_config().build_storage().unwrap();
	}
}
