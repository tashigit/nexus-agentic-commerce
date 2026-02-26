use ed25519_dalek::{SigningKey, VerifyingKey};
use rand::rngs::OsRng;

use crate::types::{AgentId, AgentInfo, Mandate, Role};

pub struct Agent {
    pub id: AgentId,
    pub role: Role,
    pub trust_score: f64,
    pub mandate: Mandate,
    pub balance: f64,
    pub daily_spent: f64,
    pub active: bool,
    signing_key: SigningKey,
    pub verifying_key: VerifyingKey,
}

impl Agent {
    pub fn new(role: Role, mandate: Mandate) -> Self {
        let signing_key = SigningKey::generate(&mut OsRng);
        let verifying_key = signing_key.verifying_key();
        let id = hex::encode(&verifying_key.to_bytes()[..8]);

        Agent {
            id,
            role,
            trust_score: 1.0,
            mandate,
            balance: 10_000.0,
            daily_spent: 0.0,
            active: true,
            signing_key,
            verifying_key,
        }
    }

    pub fn can_bid(&self, amount: f64) -> bool {
        self.active
            && amount <= self.mandate.max_bid
            && (self.daily_spent + amount) <= self.mandate.daily_spend_cap
            && amount <= self.balance
    }

    pub fn spend(&mut self, amount: f64) {
        self.balance -= amount;
        self.daily_spent += amount;
    }

    pub fn earn(&mut self, amount: f64) {
        self.balance += amount;
    }

    pub fn degrade_trust(&mut self, penalty: f64) {
        self.trust_score = (self.trust_score - penalty).max(0.0);
    }

    pub fn restore_trust(&mut self, bonus: f64) {
        self.trust_score = (self.trust_score + bonus).min(1.0);
    }

    pub fn info(&self) -> AgentInfo {
        AgentInfo {
            id: self.id.clone(),
            role: self.role,
            trust_score: self.trust_score,
            mandate: self.mandate.clone(),
            balance: self.balance,
            active: self.active,
        }
    }

    pub fn signing_key(&self) -> &SigningKey {
        &self.signing_key
    }
}

mod hex {
    pub fn encode(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{:02x}", b)).collect()
    }
}
