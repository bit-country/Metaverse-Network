// This file is part of Metaverse.Network & Bit.Country.

// The multi-metaverse governance module is inspired by frame democracy of how to store hash
// and preimages. Ref: https://github.com/paritytech/substrate/tree/master/frame/democracy

// Copyright (C) 2020-2022 Metaverse.Network & Bit.Country .
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
#[cfg(any(
	feature = "with-metaverse-runtime",
	feature = "with-pioneer-runtime"
))]
mod kusama_test_net;
// #[cfg(feature = "with-pioneer-runtime")]
// mod statemine;
