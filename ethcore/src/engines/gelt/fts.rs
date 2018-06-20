// Copyright 2015-2017 CO2KN, Inc.
// This file is part of a fork of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

// Follow the Satoshi algorithm which selects a list of proposers for blocks in an epoch.
// This selection is done based on the relative stake of the proposers.
// Based on code from Ouroboros branch in https://github.com/input-output-hk/cardano-sl/parity.

use rand::{self, SeedableRng};
use rand::distributions::{IndependentSample, Range};
use itertools::Itertools;
use ethereum_types::{U256, Address};

// TODO:
// 1. Take a random number generator instead of a seed: <R>, mut rng: R, where R: rand::Rng
// 2. Revise to make more efficient per: https://github.com/input-output-hk/cardano-sl/blob/develop/lrc/src/Pos/Lrc/Fts.hs
pub fn follow_the_satoshi<'a, I>(
	seed: I,
	epoch_balances: &[(Address, U256)],
	epoch_blocks: u64, // TODO: replace with block number
	total_coins: U256) -> Vec<Address>
	where I: IntoIterator<Item=&'a u8> {

	let seed_bytes: Vec<u8> = seed.into_iter().map(|&u| u).collect();
	let seed_slice = as_u32_seed(&seed_bytes);

	let mut rng = rand::ChaChaRng::from_seed(&seed_slice);
	let no_coins = U256::from(0);
	assert!(total_coins != no_coins, "Total amount of coin held by the validators is 0!");

	let range = Range::new(U256::zero().as_u64(), total_coins.as_u64());

	let mut coin_indices: Vec<_> = (0..epoch_blocks)
		.map(|i| (i, range.ind_sample(&mut rng)))
		.collect();

	coin_indices.sort_by_key(|&(_, r) | r);

	trace!(target: "engine", "coin_indices is {:?}", coin_indices);

	let mut max_coins = no_coins.clone().as_u64();
	let mut ci = coin_indices.iter().peekable();
	let mut block_proposers = Vec::with_capacity(epoch_blocks as usize);

	for &(stakeholder, coins) in epoch_balances {
		max_coins = max_coins + coins.as_u64();

		while let Some(&&(block, coin)) = ci.peek() {
			if coin < max_coins {
				block_proposers.push((block, stakeholder.clone()));
				ci.next();
			} else {
				break;
			}
		}
	}

	block_proposers.sort_by_key(|&(i, _)| i);

	block_proposers.into_iter().map(|(_, v)| v).collect()
}

// TODO:
// 1. Rename to better reflect behavior: Takes first 8*4=32 u8 values in a slice of u8s and turns them into a slice of 8 u32s.
// 2. Ensure sure bounds are safe with asserts and proper error handling.
fn as_u32_seed(v: &[u8]) -> Vec<u32> {
	const NUM_U32S: usize = 8;
	v.into_iter()
		.map(|&u| u)
		.chain(::std::iter::repeat(0))
		.take(NUM_U32S * 4)
		.tuples::<(_, _, _, _)>()
		.map(|c| {
			(c.3 as u32) << 24 |
				(c.2 as u32) << 16 |
				(c.1 as u32) <<  8 |
				(c.0 as u32) <<  0
		})
		.collect()
}

#[cfg(test)]
mod tests {
	use super::{follow_the_satoshi, as_u32_seed};
	use ethereum_types::{U256, Address};

	#[test]
	fn one_stakeholder_is_always_the_proposer() {
		let address = Address::from("0000000000000000000000000000000000000005");
		let balances = vec![
			(address.clone(), U256::from(10))
		];
		let seed = [1u8, 2u8, 3u8].iter();

		let result = follow_the_satoshi(seed, &balances, 3, U256::from(10));
		assert_eq!(result, vec![address.clone(), address.clone(), address.clone()]);
	}

	#[test]
	fn two_stakeholders_equal_stake() {
		let aaa = Address::from("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
		let bbb = Address::from("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb");
		let balances = vec![
			(aaa.clone(), U256::from(50)),
			(bbb.clone(), U256::from(50)),
		];
		let seed = [1u8, 2u8, 3u8].iter();

		let result = follow_the_satoshi(seed, &balances, 10, U256::from(100));
		assert_eq!(result, [
			aaa.clone(),
			bbb.clone(), bbb.clone(), bbb.clone(),
			aaa.clone(),
			bbb.clone(),
			aaa.clone(), aaa.clone(), aaa.clone(), aaa.clone()]);
	}

	#[test]
	fn two_stakeholders_skewed_stake() {
		let aaa = Address::from("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
		let bbb = Address::from("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb");
		let balances = vec![
			(aaa.clone(), U256::from(80)),
			(bbb.clone(), U256::from(20)),
		];
		let seed = [1u8, 2u8, 3u8].iter();

		let result = follow_the_satoshi(seed, &balances, 25, U256::from(100));
		assert_eq!(result, [
			aaa.clone(), aaa.clone(), aaa.clone(), aaa.clone(), aaa.clone(),
			aaa.clone(), aaa.clone(), aaa.clone(), aaa.clone(), aaa.clone(),
			aaa.clone(), aaa.cxlone(), aaa.clone(), aaa.clone(), aaa.clone(),
			aaa.clone(), aaa.clone(), aaa.clone(), aaa.clone(), bbb.clone(),
			aaa.clone(), aaa.clone(), aaa.clone(), bbb.clone(), aaa.clone()
		]);
	}

	#[test]
	fn as_u32_seed_pads_to_32() {
		let v = vec![1, 2, 3, 4, 5, 6];
		let result = as_u32_seed(&v);
		let expected = vec![67305985, 1541, 0, 0, 0, 0, 0, 0];
		assert_eq!(result, &expected[..]);
	}
}
