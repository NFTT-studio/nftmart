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
	let root_key: AccountId = hex!["fc8c5106c6d1b56df9ee507f7a7a4f88256f54bd228112520042e80e078a3d13"].into(); // 6657g5uwo58sKb1AM3rC3MsDemDxKHqzfEVL6oe6Mb6cTUW6
	let stash1: AccountId = hex!["e6f930da969042c59250db39cd9c39804139987fc3d25df2c79f90aafeaccf29"].into(); // 65apxqzr3eMQqauYha1SThyLsuP1N24TKg7VZQtiQVuAD2jj
	let controller1: AccountId = hex!["526ba4021b7d8591790659c774eba08be35ca4bc8041ada9d3784a083fca622f"].into(); // 62E3px5k1uPFJkHNeKEwNGYvj89WToyTkeck3su61Gd5bDm8
	let stash2: AccountId = hex!["94912dd08fa89e0215546359e53e1be078016a09c8ae11a50fee9f94dc305a5c"].into(); // 63in92J1aXAoRmQWKeRxe3TDLMSE2XuEU48aUEfCR3YiHiMq
	let controller2: AccountId = hex!["c86bc175eb8e0b378a6bb3737b9e5f8c320971039af885d409d234d885fb2b46"].into(); // 64tmWeytT2BgL68aWPPF5UvBEEKWtYXFKu5ssACDzQyYaRWC
	let initial_authorities: Vec<(AccountId, AccountId, GrandpaId, BabeId, ImOnlineId, AuthorityDiscoveryId)> = vec![
		(stash1, controller1,
		 hex!["f998a33fe6edd224107e9de96622fc9a65e2c739102753f349960c4533948a98"].unchecked_into(), // 661FC3RhqQVk7fWbCNNnf3vN2DCsupxAmUf3KyWrsLSZjqxV
		 hex!["a2d76db1e6ee58b2bd42bb1dbd0b8060bf0b2ed5aba92f88ead886e17bdd0f67"].unchecked_into(), // 643VgH1FyXUQepCmLXWedq5ahtoH1StFeDizVYhXMoqWQWNH
		 hex!["a2d76db1e6ee58b2bd42bb1dbd0b8060bf0b2ed5aba92f88ead886e17bdd0f67"].unchecked_into(), // 643VgH1FyXUQepCmLXWedq5ahtoH1StFeDizVYhXMoqWQWNH
		 hex!["a2d76db1e6ee58b2bd42bb1dbd0b8060bf0b2ed5aba92f88ead886e17bdd0f67"].unchecked_into(),), // 643VgH1FyXUQepCmLXWedq5ahtoH1StFeDizVYhXMoqWQWNH
		(stash2, controller2,
		 hex!["f69607190573755756d1af4f0b7adec164fe71b956d39abc93b11f0ee11a326e"].unchecked_into(), // 65wJGjzLP4MrhKaRf2Ma4BB4eqFrSFNB5CwFbRRTPpwZZK2e
		 hex!["720a4d12614cee86079a69c583600e49f73145d195981ea9e84851ddf8b4b625"].unchecked_into(), // 62wWShgtWs2BXrx7NBQKTD9pc8Qx6pVGh1Q196Vab3ARtQVm
		 hex!["720a4d12614cee86079a69c583600e49f73145d195981ea9e84851ddf8b4b625"].unchecked_into(), // 62wWShgtWs2BXrx7NBQKTD9pc8Qx6pVGh1Q196Vab3ARtQVm
		 hex!["720a4d12614cee86079a69c583600e49f73145d195981ea9e84851ddf8b4b625"].unchecked_into(),), // 62wWShgtWs2BXrx7NBQKTD9pc8Qx6pVGh1Q196Vab3ARtQVm
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
	let boot_nodes = vec![];
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
