// Copyright 2017 Parity Technologies (UK) Ltd.
// This file is part of Polkadot.

// Polkadot is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Polkadot is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Polkadot.  If not, see <http://www.gnu.org/licenses/>.

//! Primitive types for the polkadot runtime.

use codec::{Slicable, Input};
use rstd::prelude::*;
use rstd::cmp::Ordering;
use substrate_primitives;
use runtime_primitives::{self, generic, traits::BlakeTwo256};
use super::Call;

#[cfg(feature = "std")]
use substrate_primitives::bytes;

/// Block header type as expected by this runtime.
pub type Header = generic::Header<BlockNumber, BlakeTwo256, Log>;
/// Block type as expected by this runtime.
pub type Block = generic::Block<BlockNumber, BlakeTwo256, Log, AccountId, Index, Call, Signature>;
/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic = generic::UncheckedExtrinsic<AccountId, Index, Call, Signature>;
/// Extrinsic type as expected by this runtime.
pub type Extrinsic = generic::Extrinsic<AccountId, Index, Call>;

/// Something that identifies a block.
pub type BlockId = generic::BlockId<Block>;

/// A log entry in the block.
#[derive(PartialEq, Eq, Clone, Default)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize, Debug))]
pub struct Log(#[cfg_attr(feature = "std", serde(with="bytes"))] pub Vec<u8>);

impl Slicable for Log {
	fn decode<I: Input>(input: &mut I) -> Option<Self> {
		Vec::<u8>::decode(input).map(Log)
	}

	fn using_encoded<R, F: FnOnce(&[u8]) -> R>(&self, f: F) -> R {
		self.0.using_encoded(f)
	}
}

/// An index to a block.
/// 32-bits will allow for 136 years of blocks assuming 1 block per second.
/// TODO: switch to u32
pub type BlockNumber = u64;

/// Alias to Ed25519 pubkey that identifies an account on the relay chain. This will almost
/// certainly continue to be the same as the substrate's `AuthorityId`.
pub type AccountId = substrate_primitives::hash::H256;

/// The Ed25519 pub key of an session that belongs to an authority of the relay chain. This is
/// exactly equivalent to what the substrate calls an "authority".
pub type SessionKey = substrate_primitives::AuthorityId;

/// Indentifier for a chain. 32-bit should be plenty.
pub type ChainId = u32;

/// Index of a transaction in the relay chain. 32-bit should be plenty.
pub type Index = u32;

/// A hash of some data used by the relay chain.
pub type Hash = substrate_primitives::H256;

/// Alias to 512-bit hash when used in the context of a signature on the relay chain.
/// Equipped with logic for possibly "unsigned" messages.
pub type Signature = runtime_primitives::MaybeUnsigned<runtime_primitives::Ed25519Signature>;

/// A timestamp: seconds since the unix epoch.
pub type Timestamp = u64;

/// The balance of an account.
/// 128-bits (or 38 significant decimal figures) will allow for 10m currency (10^7) at a resolution
/// to all for one second's worth of an annualised 50% reward be paid to a unit holder (10^11 unit
/// denomination), or 10^18 total atomic units, to grow at 50%/year for 51 years (10^9 multiplier)
/// for an eventual total of 10^27 units (27 significant decimal figures).
/// We round denomination to 10^12 (12 sdf), and leave the other redundancy at the upper end so
/// that 32 bits may be multiplied with a balance in 128 bits without worrying about overflow.
pub type Balance = u128;

/// Parachain data types.
pub mod parachain {
	use super::*;

	/// Unique identifier of a parachain.
	#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
	#[cfg_attr(feature = "std", derive(Serialize, Debug))]
	pub struct Id(u32);

	impl From<Id> for u32 {
		fn from(x: Id) -> Self { x.0 }
	}

	impl From<u32> for Id {
		fn from(x: u32) -> Self { Id(x) }
	}

	impl Id {
		/// Convert this Id into its inner representation.
		pub fn into_inner(self) -> u32 {
			self.0
		}
	}

	impl Slicable for Id {
		fn decode<I: Input>(input: &mut I) -> Option<Self> {
			u32::decode(input).map(Id)
		}

		fn using_encoded<R, F: FnOnce(&[u8]) -> R>(&self, f: F) -> R {
			self.0.using_encoded(f)
		}
	}

	/// Identifier for a chain, either one of a number of parachains or the relay chain.
	#[derive(Copy, Clone, PartialEq)]
	#[cfg_attr(feature = "std", derive(Debug))]
	pub enum Chain {
		/// The relay chain.
		Relay,
		/// A parachain of the given index.
		Parachain(Id),
	}

	impl Slicable for Chain {
		fn decode<I: Input>(input: &mut I) -> Option<Self> {
			let disc = input.read_byte()?;

			match disc {
				0 => Some(Chain::Relay),
				1 => Some(Chain::Parachain(Slicable::decode(input)?)),
				_ => None,
			}
		}

		fn encode(&self) -> Vec<u8> {
			let mut v = Vec::new();
			match *self {
				Chain::Relay => { v.push(0); }
				Chain::Parachain(id) => {
					v.push(1u8);
					id.using_encoded(|s| v.extend(s));
				}
			}

			v
		}

		fn using_encoded<R, F: FnOnce(&[u8]) -> R>(&self, f: F) -> R {
			f(&self.encode().as_slice())
		}
	}

	/// The duty roster specifying what jobs each validator must do.
	#[derive(Clone, PartialEq)]
	#[cfg_attr(feature = "std", derive(Default, Debug))]
	pub struct DutyRoster {
		/// Lookup from validator index to chain on which that validator has a duty to validate.
		pub validator_duty: Vec<Chain>,
		/// Lookup from validator index to chain on which that validator has a duty to guarantee
		/// availability.
		pub guarantor_duty: Vec<Chain>,
	}

	impl Slicable for DutyRoster {
		fn decode<I: Input>(input: &mut I) -> Option<Self> {
			Some(DutyRoster {
				validator_duty: Slicable::decode(input)?,
				guarantor_duty: Slicable::decode(input)?,
			})
		}

		fn encode(&self) -> Vec<u8> {
			let mut v = Vec::new();

			v.extend(self.validator_duty.encode());
			v.extend(self.guarantor_duty.encode());

			v
		}

		fn using_encoded<R, F: FnOnce(&[u8]) -> R>(&self, f: F) -> R {
			f(&self.encode().as_slice())
		}
	}

	/// Extrinsic data for a parachain.
	#[derive(PartialEq, Eq, Clone)]
	#[cfg_attr(feature = "std", derive(Serialize, Debug))]
	#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
	#[cfg_attr(feature = "std", serde(deny_unknown_fields))]
	pub struct Extrinsic;

	/// Candidate parachain block.
	///
	/// https://github.com/w3f/polkadot-spec/blob/master/spec.md#candidate-para-chain-block
	#[derive(PartialEq, Eq, Clone)]
	#[cfg_attr(feature = "std", derive(Serialize, Debug))]
	#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
	#[cfg_attr(feature = "std", serde(deny_unknown_fields))]
	pub struct Candidate {
		/// The ID of the parachain this is a proposal for.
		pub parachain_index: Id,
		/// Collator's signature
		pub collator_signature: runtime_primitives::Ed25519Signature,
		/// Unprocessed ingress queue.
		///
		/// Ordered by parachain ID and block number.
		pub unprocessed_ingress: ConsolidatedIngress,
		/// Block data
		pub block: BlockData,
	}

	/// Candidate receipt type.
	#[derive(PartialEq, Eq, Clone)]
	#[cfg_attr(feature = "std", derive(Debug, Serialize))]
	#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
	#[cfg_attr(feature = "std", serde(deny_unknown_fields))]
	pub struct CandidateReceipt {
		/// The ID of the parachain this is a candidate for.
		pub parachain_index: Id,
		/// The collator's relay-chain account ID
		pub collator: super::AccountId,
		/// The head-data
		pub head_data: HeadData,
		/// Balance uploads to the relay chain.
		pub balance_uploads: Vec<(super::AccountId, u64)>,
		/// Egress queue roots.
		pub egress_queue_roots: Vec<(Id, Hash)>,
		/// Fees paid from the chain to the relay chain validators
		pub fees: u64,
	}

	impl Slicable for CandidateReceipt {
		fn encode(&self) -> Vec<u8> {
			let mut v = Vec::new();

			self.parachain_index.using_encoded(|s| v.extend(s));
			self.collator.using_encoded(|s| v.extend(s));
			self.head_data.0.using_encoded(|s| v.extend(s));
			self.balance_uploads.using_encoded(|s| v.extend(s));
			self.egress_queue_roots.using_encoded(|s| v.extend(s));
			self.fees.using_encoded(|s| v.extend(s));

			v
		}

		fn decode<I: Input>(input: &mut I) -> Option<Self> {
			Some(CandidateReceipt {
				parachain_index: Slicable::decode(input)?,
				collator: Slicable::decode(input)?,
				head_data: Slicable::decode(input).map(HeadData)?,
				balance_uploads: Slicable::decode(input)?,
				egress_queue_roots: Slicable::decode(input)?,
				fees: Slicable::decode(input)?,
			})
		}
	}

	impl CandidateReceipt {
		/// Get the blake2_256 hash
		#[cfg(feature = "std")]
		pub fn hash(&self) -> Hash {
			use runtime_primitives::traits::Hashing;
			BlakeTwo256::hash_of(self)
		}
	}

	impl PartialOrd for CandidateReceipt {
		fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
			Some(self.cmp(other))
		}
	}

	impl Ord for CandidateReceipt {
		fn cmp(&self, other: &Self) -> Ordering {
			// TODO: compare signatures or something more sane
			self.parachain_index.cmp(&other.parachain_index)
				.then_with(|| self.head_data.cmp(&other.head_data))
		}
	}

	/// Parachain ingress queue message.
	#[derive(PartialEq, Eq, Clone)]
	#[cfg_attr(feature = "std", derive(Serialize, Debug))]
	pub struct Message(#[cfg_attr(feature = "std", serde(with="bytes"))] pub Vec<u8>);

	/// Consolidated ingress queue data.
	///
	/// This is just an ordered vector of other parachains' egress queues,
	/// obtained according to the routing rules.
	#[derive(Default, PartialEq, Eq, Clone)]
	#[cfg_attr(feature = "std", derive(Serialize, Debug))]
	pub struct ConsolidatedIngress(pub Vec<(Id, Vec<Message>)>);

	/// Parachain block data.
	///
	/// contains everything required to validate para-block, may contain block and witness data
	#[derive(PartialEq, Eq, Clone)]
	#[cfg_attr(feature = "std", derive(Serialize, Debug))]
	pub struct BlockData(#[cfg_attr(feature = "std", serde(with="bytes"))] pub Vec<u8>);

	/// Parachain header raw bytes wrapper type.
	#[derive(PartialEq, Eq)]
	#[cfg_attr(feature = "std", derive(Serialize, Debug))]
	pub struct Header(#[cfg_attr(feature = "std", serde(with="bytes"))] pub Vec<u8>);

	/// Parachain head data included in the chain.
	#[derive(PartialEq, Eq, Clone, PartialOrd, Ord)]
	#[cfg_attr(feature = "std", derive(Serialize, Debug))]
	pub struct HeadData(#[cfg_attr(feature = "std", serde(with="bytes"))] pub Vec<u8>);

	/// Parachain validation code.
	#[derive(PartialEq, Eq)]
	#[cfg_attr(feature = "std", derive(Serialize, Debug))]
	pub struct ValidationCode(#[cfg_attr(feature = "std", serde(with="bytes"))] pub Vec<u8>);

	/// Activitiy bit field
	#[derive(PartialEq, Eq, Clone, Default)]
	#[cfg_attr(feature = "std", derive(Serialize, Debug))]
	pub struct Activity(#[cfg_attr(feature = "std", serde(with="bytes"))] pub Vec<u8>);

	impl Slicable for Activity {
		fn decode<I: Input>(input: &mut I) -> Option<Self> {
			Vec::<u8>::decode(input).map(Activity)
		}

		fn using_encoded<R, F: FnOnce(&[u8]) -> R>(&self, f: F) -> R {
			self.0.using_encoded(f)
		}
	}

	#[derive(Clone, Copy, PartialEq, Eq)]
	#[cfg_attr(feature = "std", derive(Debug))]
	#[repr(u8)]
	enum StatementKind {
		Candidate = 1,
		Valid = 2,
		Invalid = 3,
		Available = 4,
	}

	/// Statements which can be made about parachain candidates.
	#[derive(Clone, PartialEq, Eq)]
	#[cfg_attr(feature = "std", derive(Debug))]
	pub enum Statement {
		/// Proposal of a parachain candidate.
		Candidate(CandidateReceipt),
		/// State that a parachain candidate is valid.
		Valid(Hash),
		/// Vote to commit to a candidate.
		Invalid(Hash),
		/// Vote to advance round after inactive primary.
		Available(Hash),
	}

	impl Slicable for Statement {
		fn encode(&self) -> Vec<u8> {
			let mut v = Vec::new();
			match *self {
				Statement::Candidate(ref candidate) => {
					v.push(StatementKind::Candidate as u8);
					candidate.using_encoded(|s| v.extend(s));
				}
				Statement::Valid(ref hash) => {
					v.push(StatementKind::Valid as u8);
					hash.using_encoded(|s| v.extend(s));
				}
				Statement::Invalid(ref hash) => {
					v.push(StatementKind::Invalid as u8);
					hash.using_encoded(|s| v.extend(s));
				}
				Statement::Available(ref hash) => {
					v.push(StatementKind::Available as u8);
					hash.using_encoded(|s| v.extend(s));
				}
			}

			v
		}

		fn decode<I: Input>(value: &mut I) -> Option<Self> {
			match value.read_byte() {
				Some(x) if x == StatementKind::Candidate as u8 => {
					Slicable::decode(value).map(Statement::Candidate)
				}
				Some(x) if x == StatementKind::Valid as u8 => {
					Slicable::decode(value).map(Statement::Valid)
				}
				Some(x) if x == StatementKind::Invalid as u8 => {
					Slicable::decode(value).map(Statement::Invalid)
				}
				Some(x) if x == StatementKind::Available as u8 => {
					Slicable::decode(value).map(Statement::Available)
				}
				_ => None,
			}
		}
	}
}