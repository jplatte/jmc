use std::{fmt, ops::Deref, sync::Arc};

use paste::paste;
use ruma::{EventId, RoomId, TransactionId, UserId};

macro_rules! id_arc {
    ($inner:ident) => {
        paste! { id_arc!($inner, [<$inner Arc>]); }
    };
    ($inner:ident, $name:ident) => {
        #[derive(Clone, PartialEq, Eq, PartialOrd, Ord, druid::Data)]
        pub struct $name(#[data(eq)] Arc<$inner>);

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
            type Target = Arc<$inner>;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl From<&$inner> for $name {
            fn from(room_id: &$inner) -> Self {
                Self(room_id.to_owned().into())
            }
        }

        impl From<&Box<$inner>> for $name {
            fn from(room_id: &Box<$inner>) -> Self {
                Self(room_id.deref().into())
            }
        }

        impl From<Box<$inner>> for $name {
            fn from(room_id: Box<$inner>) -> Self {
                Self(room_id.into())
            }
        }

        impl From<Arc<$inner>> for $name {
            fn from(arc: Arc<$inner>) -> Self {
                Self(arc)
            }
        }

        impl From<&Arc<$inner>> for $name {
            fn from(arc: &Arc<$inner>) -> Self {
                Self(arc.clone())
            }
        }
    };
}

id_arc!(EventId);
id_arc!(RoomId);
id_arc!(TransactionId);
id_arc!(UserId);

impl TransactionIdArc {
    pub fn new() -> Self {
        TransactionId::new().into()
    }
}
