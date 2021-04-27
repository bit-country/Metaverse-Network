// This file is part of Bit.Country.

// Copyright (C) 2020-2021 Bit.Country.
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

#![cfg(test)]

use super::*;
use frame_support::{assert_noop, assert_ok};
use mock::{Event, *};

#[test]
// Private test continuum pallet to ensure mocking and unit tests working
fn test() {
    ExtBuilder::default().build().execute_with(|| {
        assert_eq!(0, 0)
    });
}

#[test]
fn find_neighborhood_spot_is_work() {
    ExtBuilder::default().build().execute_with(|| {
        let continuum_spot = ContinuumSpot {
            x: 0,
            y: 0,
            country: COUNTRY_ID,
        };

        let correct_neighbors = vec![
            (-1, -1),
            (-1, 0),
            (-1, 1),
            (0, -1),
            (0, 1),
            (1, -1),
            (1, 0),
            (1, 1),
        ];

        let neighbors = continuum_spot.find_neighbour();
        let total_matching = neighbors.iter().zip(&correct_neighbors).filter(|&(a, b)| a.0 == b.0 && a.1 == b.1).count();

        assert_eq!(8, total_matching)
    })
}

#[test]
fn