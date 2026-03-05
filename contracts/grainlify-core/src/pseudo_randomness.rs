//! Deterministic pseudo-randomness helpers for on-chain selection flows.
//!
//! # Design
//! - Fully deterministic and replayable from public inputs.
//! - Seeded from domain/context bytes + caller-provided external seed.
//! - Candidate selection uses per-candidate score hashing to avoid index/order bias.
//!
//! # Security Trade-offs
//! - This is **not** true randomness; validators/submitters can still influence
//!   timing-dependent context.
//! - A caller controlling both context and external seed can brute-force outcomes.
//! - To reduce bias, consumers should include hard-to-predict values (e.g. commit
//!   reveals, prior state roots, delayed reveal windows) and publish seed sources.
//!
//! # Adversarial Examples
//! - **Seed grinding**: attacker tries many external seeds off-chain until a
//!   preferred winner appears.
//! - **Timing bias**: attacker submits only when ledger metadata favors outcome.
//! - **Candidate stuffing**: attacker adds sybil addresses to increase odds.
//!
//! This helper mitigates order-manipulation by scoring each candidate directly
//! instead of selecting by `hash % n`.

use core::cmp::Ordering;
use soroban_sdk::xdr::ToXdr;
use soroban_sdk::{Address, Bytes, BytesN, Env, Symbol, Vec};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeterministicSelection {
    pub index: u32,
    pub seed_hash: BytesN<32>,
    pub winner_score: BytesN<32>,
}

fn cmp_hash(env: &Env, a: &BytesN<32>, b: &BytesN<32>) -> Ordering {
    let ax = a.clone().to_xdr(env);
    let bx = b.clone().to_xdr(env);
    let mut i: u32 = 0;
    while i < ax.len() && i < bx.len() {
        let av = ax.get(i).unwrap();
        let bv = bx.get(i).unwrap();
        if av < bv {
            return Ordering::Less;
        }
        if av > bv {
            return Ordering::Greater;
        }
        i += 1;
    }
    ax.len().cmp(&bx.len())
}

fn build_seed_hash(
    env: &Env,
    domain: &Symbol,
    context: &Bytes,
    external_seed: &BytesN<32>,
) -> BytesN<32> {
    let mut seed_material = Bytes::new(env);
    seed_material.append(&domain.to_xdr(env));
    seed_material.append(context);
    seed_material.append(&external_seed.clone().to_xdr(env));
    env.crypto().sha256(&seed_material).into()
}

/// Derive a deterministic winner index from candidates + seed material.
///
/// Returns `None` when `candidates` is empty.
pub fn derive_selection(
    env: &Env,
    domain: &Symbol,
    context: &Bytes,
    external_seed: &BytesN<32>,
    candidates: &Vec<Address>,
) -> Option<DeterministicSelection> {
    if candidates.is_empty() {
        return None;
    }

    let seed_hash = build_seed_hash(env, domain, context, external_seed);

    let mut best_idx: u32 = 0;
    let mut best_score: Option<BytesN<32>> = None;
    let mut i: u32 = 0;

    while i < candidates.len() {
        let candidate = candidates.get(i).unwrap();
        let mut score_material = Bytes::new(env);
        score_material.append(&seed_hash.clone().to_xdr(env));
        score_material.append(&candidate.to_xdr(env));
        let score: BytesN<32> = env.crypto().sha256(&score_material).into();

        match &best_score {
            None => {
                best_score = Some(score);
                best_idx = i;
            }
            Some(current_best) => {
                if cmp_hash(env, &score, current_best) == Ordering::Greater {
                    best_score = Some(score);
                    best_idx = i;
                }
            }
        }
        i += 1;
    }

    Some(DeterministicSelection {
        index: best_idx,
        seed_hash,
        winner_score: best_score.unwrap(),
    })
}
