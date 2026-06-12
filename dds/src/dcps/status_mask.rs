use crate::infrastructure::status::StatusKind;

/// A mask representation of [`StatusKind`] variants stored as bits in a `u16`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct StatusMask(u16);

impl StatusMask {
    /// Checks if the status kind is enabled in this mask.
    pub fn is_enabled(&self, status: &StatusKind) -> bool {
        (self.0 & Self::status_kind_bit(status)) != 0
    }

    const fn status_kind_bit(status: &StatusKind) -> u16 {
        match status {
            StatusKind::InconsistentTopic => 1 << 0,
            StatusKind::OfferedDeadlineMissed => 1 << 1,
            StatusKind::RequestedDeadlineMissed => 1 << 2,
            StatusKind::OfferedIncompatibleQos => 1 << 3,
            StatusKind::RequestedIncompatibleQos => 1 << 4,
            StatusKind::SampleLost => 1 << 5,
            StatusKind::SampleRejected => 1 << 6,
            StatusKind::DataOnReaders => 1 << 7,
            StatusKind::DataAvailable => 1 << 8,
            StatusKind::LivelinessLost => 1 << 9,
            StatusKind::LivelinessChanged => 1 << 10,
            StatusKind::PublicationMatched => 1 << 11,
            StatusKind::SubscriptionMatched => 1 << 12,
        }
    }
}

impl<'a> core::iter::FromIterator<&'a StatusKind> for StatusMask {
    fn from_iter<T: IntoIterator<Item = &'a StatusKind>>(iter: T) -> Self {
        let mut mask = 0;
        for status in iter {
            mask |= Self::status_kind_bit(status);
        }
        Self(mask)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_iterator() {
        let mask: StatusMask = [StatusKind::SampleLost, StatusKind::PublicationMatched]
            .iter()
            .collect();

        assert!(mask.is_enabled(&StatusKind::SampleLost));
        assert!(mask.is_enabled(&StatusKind::PublicationMatched));
        assert!(!mask.is_enabled(&StatusKind::DataAvailable));
    }
}
