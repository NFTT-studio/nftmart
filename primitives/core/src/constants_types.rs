// This file is part of Substrate.

// Copyright (C) 2017-2021 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! constants and types

/// Balance type.
pub type Balance = u128;

/// A unit balance
pub const ACCURACY: Balance = 1_000_000_000_000u128;

/// A type for ORML currency Id
pub type CurrencyIdType = u32;

/// Signed version of Balance
pub type Amount = i128;

/// Native currency
pub const NATIVE_CURRENCY_ID: CurrencyIdType = 1;

/// Type used for expressing timestamp.
pub type Moment = u64;
