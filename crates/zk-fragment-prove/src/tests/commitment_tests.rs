//! Comprehensive commitment scheme tests

#[cfg(test)]
mod tests {
    use zk_fragment_prove::commitment::*;

    #[test]
    fn test_poseidon_hash_binding() {
        let blinding = [1u8; 32];
        let c1 = poseidon_hash(42u64, &blinding);
        let c2 = poseidon_hash(43u64, &blinding);
        
        assert_ne!(c1.value, c2.value, "Different values must produce different commitments");
    }

    #[test]
    fn test_poseidon_hash_hiding() {
        let value = 42u64;
        let c1 = poseidon_hash(value, &[1u8; 32]);
        let c2 = poseidon_hash(value, &[2u8; 32]);
        
        assert_ne!(c1.value, c2.value, "Different blindings must produce different commitments");
    }

    #[test]
    fn test_commitment_verification_valid() {
        let value = 42u64;
        let blinding = [1u8; 32];
        let commitment = poseidon_hash(value, &blinding);
        let opening = CommitmentOpening { value, blinding };
        
        assert!(verify_commitment(&commitment, &opening));
    }

    #[test]
    fn test_commitment_verification_invalid_value() {
        let blinding = [1u8; 32];
        let commitment = poseidon_hash(42u64, &blinding);
        let wrong_opening = CommitmentOpening { value: 43u64, blinding };
        
        assert!(!verify_commitment(&commitment, &wrong_opening));
    }

    #[test]
    fn test_commitment_verification_invalid_blinding() {
        let value = 42u64;
        let commitment = poseidon_hash(value, &[1u8; 32]);
        let wrong_opening = CommitmentOpening { value, blinding: [2u8; 32] };
        
        assert!(!verify_commitment(&commitment, &wrong_opening));
    }

    #[test]
    fn test_blinding_factor_derivation_deterministic() {
        let b1 = derive_blinding_factor(1, 100, &[3u8; 32]);
        let b2 = derive_blinding_factor(1, 100, &[3u8; 32]);
        
        assert_eq!(b1, b2, "Same inputs must produce same blinding");
    }

    #[test]
    fn test_blinding_factor_derivation_different_fragments() {
        let b1 = derive_blinding_factor(1, 100, &[3u8; 32]);
        let b2 = derive_blinding_factor(2, 100, &[3u8; 32]);
        
        assert_ne!(b1, b2, "Different fragments must produce different blindings");
    }

    #[test]
    fn test_generator_batch_commitments() {
        let config = CommitmentConfig::new(1);
        let gen = CommitmentGenerator::new(config);
        
        let wires = vec![(100, 42u64), (101, 43u64), (102, 44u64)];
        let commits = gen.generate_commitments(&wires);
        
        assert_eq!(commits.len(), 3);
        for (i, (wid, _)) in wires.iter().enumerate() {
            assert_eq!(commits[i].wire_id, *wid);
        }
    }

    #[test]
    fn test_verifier_multiple_commitments() {
        let blinding = [1u8; 32];
        let c1 = poseidon_hash(42u64, &blinding);
        let c2 = poseidon_hash(43u64, &blinding);
        
        let mut verifier = CommitmentVerifier::new();
        verifier.register_commitment(100, c1);
        verifier.register_commitment(101, c2);
        
        let openings = vec![
            (100, CommitmentOpening { value: 42, blinding }),
            (101, CommitmentOpening { value: 43, blinding }),
        ];
        
        assert!(verifier.verify_openings(&openings).is_ok());
    }
}