use codec::{Decode, Encode};
use scale_info::TypeInfo;

#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone, TypeInfo)]
pub enum Choice {
	Yes,
	No,
}
