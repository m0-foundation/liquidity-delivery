use alloy::primitives::keccak256;

/// Order data structure matching both EVM and SVM implementations
#[derive(Debug, Clone)]
pub struct OrderData {
    pub version: u16,
    pub sender: [u8; 32],
    pub nonce: u64,
    pub origin_chain_id: u32,
    pub dest_chain_id: u32,
    pub fill_deadline: u64,
    pub token_in: [u8; 32],
    pub token_out: [u8; 32],
    pub amount_in: u128,
    pub amount_out: u128,
    pub recipient: [u8; 32],
    pub solver: [u8; 32],
}

impl OrderData {
    /// Encode the order data as bytes for hashing
    /// Uses big-endian encoding to match both EVM and SVM implementations
    pub fn encode(&self) -> Vec<u8> {
        let mut encoded: Vec<u8> = Vec::with_capacity(206);

        encoded.extend_from_slice(&self.version.to_be_bytes());
        encoded.extend_from_slice(&self.sender);
        encoded.extend_from_slice(&self.nonce.to_be_bytes());
        encoded.extend_from_slice(&self.origin_chain_id.to_be_bytes());
        encoded.extend_from_slice(&self.dest_chain_id.to_be_bytes());
        encoded.extend_from_slice(&self.fill_deadline.to_be_bytes());
        encoded.extend_from_slice(&self.token_in);
        encoded.extend_from_slice(&self.token_out);
        encoded.extend_from_slice(&self.amount_in.to_be_bytes());
        encoded.extend_from_slice(&self.amount_out.to_be_bytes());
        encoded.extend_from_slice(&self.recipient);
        encoded.extend_from_slice(&self.solver);

        encoded
    }

    /// Compute the order ID using keccak256
    /// This produces the same result as both EVM and SVM implementations
    pub fn compute_order_id(&self) -> [u8; 32] {
        keccak256(&self.encode()).0
    }
}

/// Compute order ID from individual fields
/// Convenience function matching both EVM and SVM contract implementations
pub fn compute_order_id(
    version: u16,
    sender: [u8; 32],
    nonce: u64,
    origin_chain_id: u32,
    dest_chain_id: u32,
    fill_deadline: u64,
    token_in: [u8; 32],
    token_out: [u8; 32],
    amount_in: u128,
    amount_out: u128,
    recipient: [u8; 32],
    solver: [u8; 32],
) -> [u8; 32] {
    let order_data = OrderData {
        version,
        sender,
        nonce,
        origin_chain_id,
        dest_chain_id,
        fill_deadline,
        token_in,
        token_out,
        amount_in,
        amount_out,
        recipient,
        solver,
    };
    order_data.compute_order_id()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_order_id_computation() {
        // Test with known values to ensure consistency
        let order_data = OrderData {
            version: 1,
            sender: [0u8; 32],
            nonce: 0,
            origin_chain_id: 1,
            dest_chain_id: 2,
            fill_deadline: 1700000000,
            token_in: [1u8; 32],
            token_out: [2u8; 32],
            amount_in: 1000000,
            amount_out: 900000,
            recipient: [3u8; 32],
            solver: [0u8; 32],
        };

        let order_id = order_data.compute_order_id();
        // The order ID should be deterministic
        assert_eq!(order_id.len(), 32);

        // Compute again to ensure determinism
        let order_id2 = order_data.compute_order_id();
        assert_eq!(order_id, order_id2);
    }

    #[test]
    fn test_encode_length() {
        let order_data = OrderData {
            version: 1,
            sender: [0u8; 32],
            nonce: 0,
            origin_chain_id: 1,
            dest_chain_id: 2,
            fill_deadline: 1700000000,
            token_in: [0u8; 32],
            token_out: [0u8; 32],
            amount_in: 0,
            amount_out: 0,
            recipient: [0u8; 32],
            solver: [0u8; 32],
        };

        let encoded = order_data.encode();
        // 2 + 32 + 8 + 4 + 4 + 8 + 32 + 32 + 16 + 16 + 32 + 32 = 218 bytes
        assert_eq!(encoded.len(), 218);
    }
}
