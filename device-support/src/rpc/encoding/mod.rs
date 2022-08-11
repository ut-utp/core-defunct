mod postcard;

pub use self::postcard::{PostcardEncode, DynFifoBorrow};
pub use self::postcard::PostcardDecode;

pub use ::postcard::ser_flavors::{Cobs, Slice};
