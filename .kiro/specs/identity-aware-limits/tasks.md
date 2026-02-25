# Implementation Plan

- [x] 1. Set up identity-aware limits module structure
  - Create new Rust module for identity types and functions
  - Add identity-related dependencies to Cargo.toml
  - Define error types for identity verification
  - _Requirements: 1.1, 2.1, 6.1-6.5_

- [ ] 2. Implement core identity data structures
  - [x] 2.1 Define IdentityClaim, IdentityTier, and AddressIdentity types
    - Create contracttype structs for claim and identity data
    - Implement IdentityTier enum with Unverified, Basic, Verified, Premium
    - Add DataKey variants for identity storage
    - _Requirements: 2.1, 7.1_

  - [ ]* 2.2 Write property test for claim structure completeness
    - **Property 5: Claims contain all required fields**
    - **Validates: Requirements 2.1**

  - [x] 2.3 Define TierLimits and RiskThresholds configuration structures
    - Create contracttype structs for limit configuration
    - Add storage keys for tier limits and risk thresholds
    - _Requirements: 4.1-4.5, 5.1-5.5_

- [ ] 3. Implement claim serialization and signature verification
  - [x] 3.1 Implement claim serialization using XDR encoding
    - Create deterministic serialization function for claims
    - Ensure consistent byte ordering for signature verification
    - _Requirements: 2.5_

  - [ ]* 3.2 Write property test for serialization round-trip
    - **Property 9: Claim serialization round-trip consistency**
    - **Validates: Requirements 2.5**

  - [x] 3.3 Implement Ed25519 signature verification
    - Create verify_claim_signature function
    - Verify signature against authorized issuer public key
    - _Requirements: 1.3, 2.2, 3.1_

  - [ ]* 3.4 Write property test for signature validity
    - **Property 6: Claim signatures are cryptographically valid**
    - **Validates: Requirements 2.2**

  - [ ]* 3.5 Write property test for signature invalidation on modification
    - **Property 7: Modified claim data invalidates signature**
    - **Validates: Requirements 2.3**

- [ ] 4. Implement issuer authorization management
  - [x] 4.1 Create set_authorized_issuer admin function
    - Implement function to add/remove authorized issuers
    - Require admin authentication
    - Store issuer authorization in persistent storage
    - _Requirements: 1.1, 1.2, 1.5_

  - [ ]* 4.2 Write property test for issuer authorization updates
    - **Property 1: Issuer authorization updates replace previous values**
    - **Validates: Requirements 1.2**

  - [ ]* 4.3 Write property test for authorized issuer list consistency
    - **Property 3: Authorized issuer list maintains consistency**
    - **Validates: Requirements 1.4**

  - [ ]* 4.4 Write property test for removed issuer rejection
    - **Property 4: Removed issuers cannot sign valid claims**
    - **Validates: Requirements 1.5**

  - [x] 4.5 Emit events for issuer management actions
    - Create event for issuer addition/removal
    - Include issuer public key and action in event
    - _Requirements: 8.5_

  - [ ]* 4.6 Write property test for issuer management events
    - **Property 29: Issuer management emits events**
    - **Validates: Requirements 8.5**

- [ ] 5. Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [ ] 6. Implement claim submission and verification
  - [ ] 6.1 Create submit_identity_claim function
    - Verify claim signature against authorized issuer
    - Check claim expiry timestamp
    - Validate claim format and fields
    - Store identity data for address
    - _Requirements: 3.1-3.5_

  - [ ]* 6.2 Write property test for valid claim acceptance
    - **Property 10: Valid claim signatures are accepted**
    - **Validates: Requirements 3.1**

  - [ ]* 6.3 Write property test for invalid signature rejection
    - **Property 11: Invalid signatures preserve current state**
    - **Validates: Requirements 3.2**

  - [ ]* 6.4 Write property test for expired claim rejection
    - **Property 12: Expired claims are rejected**
    - **Validates: Requirements 3.3**

  - [ ]* 6.5 Write property test for identity data updates
    - **Property 13: Valid claims update identity data**
    - **Validates: Requirements 3.4**

  - [ ]* 6.6 Write property test for claim replacement
    - **Property 14: New claims replace previous claims**
    - **Validates: Requirements 3.5**

  - [ ] 6.7 Implement claim expiry validation
    - Create is_claim_expired helper function
    - Check current timestamp against expiry
    - _Requirements: 2.4, 3.3_

  - [ ]* 6.8 Write property test for future expiry requirement
    - **Property 8: Claim expiry must be in the future**
    - **Validates: Requirements 2.4**

  - [ ] 6.9 Emit events for claim submission results
    - Create event for valid claim submission
    - Create event for claim rejection
    - Include address, tier, risk score, and reason in events
    - _Requirements: 8.1, 8.2_

  - [ ]* 6.10 Write property tests for claim submission events
    - **Property 25: Valid claim submission emits event**
    - **Property 26: Rejected claims emit rejection event**
    - **Validates: Requirements 8.1, 8.2**

- [ ] 7. Implement tier limit configuration
  - [x] 7.1 Create set_tier_limits admin function
    - Implement function to configure limits for each tier
    - Require admin authentication
    - Store tier limits in persistent storage
    - _Requirements: 4.1-4.5_

  - [x] 7.2 Create set_risk_thresholds admin function
    - Implement function to configure risk thresholds and multipliers
    - Require admin authentication
    - Store risk thresholds in persistent storage
    - _Requirements: 5.1-5.5_

- [ ] 8. Implement limit calculation and enforcement
  - [ ] 8.1 Create calculate_effective_limit function
    - Retrieve address identity data
    - Get tier-based limit from configuration
    - Calculate risk-adjusted limit if applicable
    - Return minimum of tier limit and risk-adjusted limit
    - _Requirements: 5.4, 7.4_

  - [ ]* 8.2 Write property test for tier-based limit enforcement
    - **Property 15: Tier-based limits are enforced correctly**
    - **Validates: Requirements 4.2, 4.3, 4.4**

  - [ ]* 8.3 Write property test for high risk score limit reduction
    - **Property 18: High risk scores reduce limits**
    - **Validates: Requirements 5.2**

  - [ ]* 8.4 Write property test for low risk score standard limits
    - **Property 19: Low risk scores use standard limits**
    - **Validates: Requirements 5.3**

  - [ ]* 8.5 Write property test for effective limit minimum calculation
    - **Property 20: Effective limit is minimum of tier and risk-adjusted limits**
    - **Validates: Requirements 5.4**

  - [ ]* 8.6 Write property test for risk score update effects
    - **Property 21: Updated risk scores immediately affect limits**
    - **Validates: Requirements 5.5**

  - [ ] 8.7 Create enforce_transaction_limit function
    - Check transaction amount against effective limit
    - Return error if amount exceeds limit
    - Include limit and attempted amount in error
    - _Requirements: 4.5, 6.5_

  - [ ]* 8.8 Write property test for over-limit transaction rejection
    - **Property 16: Over-limit transactions are rejected**
    - **Validates: Requirements 4.5**

  - [ ] 8.9 Emit events for limit enforcement
    - Create event for limit check results
    - Include address, amount, limit, and result in event
    - _Requirements: 8.4_

  - [ ]* 8.10 Write property test for limit enforcement events
    - **Property 28: Limit enforcement emits event**
    - **Validates: Requirements 8.4**

- [ ] 9. Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [ ] 10. Integrate limit enforcement into escrow operations
  - [ ] 10.1 Add limit checks to lock_funds function
    - Call enforce_transaction_limit before locking funds
    - Reject transactions that exceed limits
    - _Requirements: 4.1-4.5_

  - [ ] 10.2 Add limit checks to release_funds function
    - Call enforce_transaction_limit before releasing funds
    - Reject releases that exceed limits
    - _Requirements: 4.1-4.5_

  - [ ]* 10.3 Write integration tests for escrow limit enforcement
    - Test lock_funds with various tier limits
    - Test release_funds with various tier limits
    - Test rejection of over-limit operations
    - _Requirements: 4.1-4.5_

- [ ] 11. Implement query functions
  - [ ] 11.1 Create get_address_identity function
    - Retrieve identity data for address
    - Return default unverified tier if no claim exists
    - Check expiry and return unverified if expired
    - _Requirements: 7.1, 7.2, 7.3_

  - [ ]* 11.2 Write property test for identity query
    - **Property 22: Query returns stored identity data**
    - **Validates: Requirements 7.1**

  - [ ] 11.3 Create get_effective_limit function
    - Call calculate_effective_limit for address
    - Return the calculated limit
    - _Requirements: 7.4_

  - [ ]* 11.4 Write property test for effective limit query
    - **Property 23: Effective limit query matches calculation**
    - **Validates: Requirements 7.4**

  - [ ] 11.5 Create is_claim_valid query function
    - Check if address has non-expired claim
    - Return validity status
    - _Requirements: 7.5_

  - [ ]* 11.6 Write property test for claim validity check
    - **Property 24: Claim validity check reflects expiry status**
    - **Validates: Requirements 7.5**

  - [ ] 11.7 Emit event when expired claim is detected
    - Create event for claim expiry detection
    - Emit during transaction attempts with expired claims
    - _Requirements: 8.3_

  - [ ]* 11.8 Write property test for expiry detection events
    - **Property 27: Expired claims emit expiry event**
    - **Validates: Requirements 8.3**

- [ ] 12. Implement off-chain helper module in Go
  - [ ] 12.1 Create identity claims package structure
    - Create backend/internal/identity/claims.go
    - Define IdentityClaim and IdentityTier types
    - _Requirements: 9.1_

  - [ ] 12.2 Implement CreateClaim function
    - Generate claim with address, tier, risk score, and expiry
    - Validate input parameters
    - _Requirements: 9.1_

  - [ ]* 12.3 Write property test for off-chain claim generation
    - **Property 30: Off-chain helper generates valid claim structure**
    - **Validates: Requirements 9.1**

  - [ ] 12.3 Implement SerializeClaim function
    - Use same XDR encoding as on-chain contract
    - Ensure deterministic byte ordering
    - _Requirements: 9.4_

  - [ ]* 12.4 Write property test for serialization consistency
    - **Property 33: Off-chain and on-chain serialization consistency**
    - **Validates: Requirements 9.4**

  - [ ] 12.5 Implement SignClaim function
    - Sign serialized claim with Ed25519 private key
    - Return 64-byte signature
    - _Requirements: 9.2_

  - [ ]* 12.6 Write property test for off-chain signature compatibility
    - **Property 31: Off-chain signature is on-chain compatible**
    - **Validates: Requirements 9.2**

  - [ ] 12.7 Implement VerifyClaim function
    - Verify signature using Ed25519 public key
    - Use same logic as on-chain verification
    - _Requirements: 9.3_

  - [ ]* 12.8 Write property test for verification consistency
    - **Property 32: Off-chain and on-chain verification consistency**
    - **Validates: Requirements 9.3**

  - [ ] 12.9 Create test utilities for claim generation
    - Implement helper to generate test claims with valid signatures
    - Provide utilities for different tiers and risk scores
    - _Requirements: 9.5_

  - [ ]* 12.10 Write property test for test utility claims
    - **Property 34: Test utilities generate valid claims**
    - **Validates: Requirements 9.5**

- [ ] 13. Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [ ] 14. Write comprehensive property-based tests
  - [ ]* 14.1 Write property test for valid claim acceptance
    - **Property 35: Valid claims are accepted**
    - **Validates: Requirements 10.1**

  - [ ]* 14.2 Write property test for invalid signature rejection
    - **Property 36: Invalid signatures are rejected**
    - **Validates: Requirements 10.2**

  - [ ]* 14.3 Write property test for expired claim rejection
    - **Property 37: Expired claims are rejected**
    - **Validates: Requirements 10.3**

  - [ ]* 14.4 Write property test for tier transitions
    - **Property 38: Tier transitions update limits correctly**
    - **Validates: Requirements 10.4**

  - [ ]* 14.5 Write property test for limit enforcement
    - **Property 39: Limit enforcement rejects over-limit transactions**
    - **Validates: Requirements 10.5**

- [ ] 15. Write unit tests for edge cases
  - [ ]* 15.1 Write unit test for unverified user default tier
    - Test that addresses with no claims have unverified tier
    - Test that unverified tier has lowest limits
    - _Requirements: 4.1, 7.2_

  - [ ]* 15.2 Write unit test for expired claim reversion
    - Test that expired claims revert to unverified tier
    - Test that queries return unverified for expired claims
    - _Requirements: 7.3_

  - [ ]* 15.3 Write unit tests for error messages
    - Test InvalidSignature error message
    - Test ClaimExpired error message
    - Test UnauthorizedIssuer error message
    - Test InvalidClaimFormat error message
    - Test TransactionExceedsLimit error message
    - _Requirements: 6.1-6.5_

  - [ ]* 15.4 Write unit tests for boundary values
    - Test risk score boundaries (0, 100)
    - Test transaction amount at limit
    - Test transaction amount just over limit
    - _Requirements: 4.5, 5.1_

- [ ] 16. Final checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.
