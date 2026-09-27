use std::io::Write;

use crate::state::State;

pub fn write_accounts_to_csv<W: Write>(state: &State, writer: W) -> Result<(), csv::Error> {
    let mut csv_writer = csv::Writer::from_writer(writer);

    for account in state.accounts() {
        csv_writer.serialize(account)?;
    }

    csv_writer.flush()?;
    Ok(())
}
