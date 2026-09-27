use rust_decimal::Decimal;

#[derive(Debug, serde::Serialize)]
pub struct Account {
    client_id: u16,
    available: Decimal,
    held: Decimal,
    locked: bool,
}

impl Account {

    // constructor
    pub fn new(client_id: u16) -> Self {
        Self {
            client_id,
            available: Decimal::ZERO,
            held: Decimal::ZERO,
            locked: false,
        }
    }

    pub fn deposit(&mut self, amount: &Decimal) {
        self.available += amount;
    }

    pub fn withdraw(&mut self, amount: &Decimal) {
        self.available -= amount;
    }

    pub fn dispute(&mut self, amount_disputed: &Decimal) {
        self.available -= amount_disputed;
        self.held += amount_disputed;
    }

    pub fn resolve(&mut self, amount_resolved: &Decimal) {
        self.held -= amount_resolved;
        self.available += amount_resolved;
    }

    pub fn chargeback(&mut self, amount_chargedback: &Decimal) {
        self.held -= amount_chargedback;
        self.locked = true;
    }

}