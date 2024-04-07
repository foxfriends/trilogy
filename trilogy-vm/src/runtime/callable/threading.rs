#[cfg(not(feature = "multithread"))]
pub trait Threading {}
#[cfg(not(feature = "multithread"))]
impl<T> Threading for T {}

#[cfg(feature = "multithread")]
pub trait Threading: Send + Sync {}
#[cfg(feature = "multithread")]
impl<T: Send + Sync> Threading for T {}
