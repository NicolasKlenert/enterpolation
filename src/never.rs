///! Currently, the never type is a night-only feature and cannot be used in stable builds.
///! This enum is an alternative implementation of the never type which is used in the meantime.

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Never {}

#[cfg(test)]
mod tests {
    use std::mem::size_of;

    use super::*;

    #[test]
    fn size_of_never_is_zero() {
        assert_eq!(0, size_of::<Never>());
    }
}
