pub(crate) trait FromProto<T>: Sized {
    type Error;
    fn from_proto(proto: T) -> Result<Self, Self::Error>;
}

pub(crate) trait IntoProto<T>: Sized {
    fn into_proto(self) -> T;
}
