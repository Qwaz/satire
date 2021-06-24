/*!
Custom Snafu error printer
*/

use std::error::Error as StdError;

pub struct Report(Box<dyn StdError>);

impl std::fmt::Debug for Report {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.0)?;

        if let Some(source) = self.0.source() {
            writeln!(f, "\nCaused by:")?;
            for (i, e) in std::iter::successors(Some(source), |e| e.source()).enumerate() {
                writeln!(f, "  {}: {}", i, e)?;
            }
        }

        Ok(())
    }
}

impl<E: Into<Box<dyn StdError>>> From<E> for Report {
    fn from(e: E) -> Self {
        Report(e.into())
    }
}
