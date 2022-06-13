use std::{fmt, ops::Deref};

macro_rules! id_type {
    ($name:ident, $owned:ident) => {
        #[derive(Clone, PartialEq, Eq, PartialOrd, Ord, druid::Data)]
        pub struct $name(#[data(eq)] ruma::$owned);

        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.fmt(f)
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.fmt(f)
            }
        }

        impl Deref for $name {
            type Target = ruma::$name;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl From<&ruma::$name> for $name {
            fn from(room_id: &ruma::$name) -> Self {
                Self(room_id.to_owned())
            }
        }

        impl From<&ruma::$owned> for $name {
            fn from(room_id: &ruma::$owned) -> Self {
                Self(room_id.to_owned())
            }
        }

        impl From<ruma::$owned> for $name {
            fn from(room_id: ruma::$owned) -> Self {
                Self(room_id)
            }
        }
    };
}

id_type!(EventId, OwnedEventId);
id_type!(RoomId, OwnedRoomId);
id_type!(TransactionId, OwnedTransactionId);
id_type!(UserId, OwnedUserId);

impl TransactionId {
    pub fn new() -> Self {
        ruma::TransactionId::new().into()
    }
}
