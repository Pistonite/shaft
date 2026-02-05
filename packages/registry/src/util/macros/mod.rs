/// Macros for generating the package definition
mod generator;
pub(crate) use generator::*;
/// Macros for implementing verify() of the package
mod verify;
pub(crate) use verify::*;

macro_rules! test_config {
    ($IDENT:ident) => {
        #[cfg(test)]
        mod test_config {
            #[test]
            fn parse_default_config() -> cu::Result<()> {
                super::$IDENT.load_default()?;
                Ok(())
            }
        }
    };
}
pub(crate) use test_config;


