#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Flag {
    Complete,
    Incomplete,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_posting_flag_equality() {
        assert_eq!(Flag::Complete, Flag::Complete);
        assert_eq!(Flag::Incomplete, Flag::Incomplete);
        assert_ne!(Flag::Complete, Flag::Incomplete);
    }

    #[test]
    fn test_posting_flag_clone() {
        let flag1 = Flag::Complete;
        let flag2 = flag1.clone();
        assert_eq!(flag1, flag2);
    }

    #[test]
    fn test_posting_flag_ordering() {
        let complete = Flag::Complete;
        let incomplete = Flag::Incomplete;

        // Test ordering is consistent
        assert!(complete <= complete);
        assert!(incomplete <= incomplete);
    }
}
