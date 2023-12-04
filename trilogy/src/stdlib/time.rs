#[trilogy_derive::module(crate_name=crate)]
pub mod time {
    use crate::{Result, Runtime};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[derive(Clone)]
    pub struct Instant(std::time::Instant);

    #[trilogy_derive::module(crate_name=crate)]
    impl Instant {
        #[trilogy_derive::proc(crate_name=crate)]
        pub fn elapsed_ns(self, rt: Runtime) -> Result<()> {
            rt.r#return(self.0.elapsed().as_nanos())
        }

        #[trilogy_derive::proc(crate_name=crate)]
        pub fn elapsed_us(self, rt: Runtime) -> Result<()> {
            rt.r#return(self.0.elapsed().as_micros())
        }

        #[trilogy_derive::proc(crate_name=crate)]
        pub fn elapsed_ms(self, rt: Runtime) -> Result<()> {
            rt.r#return(self.0.elapsed().as_millis())
        }

        #[trilogy_derive::proc(crate_name=crate)]
        pub fn elapsed_s(self, rt: Runtime) -> Result<()> {
            rt.r#return(self.0.elapsed().as_secs())
        }
    }

    #[trilogy_derive::proc(crate_name=crate)]
    pub fn now_ns(rt: Runtime) -> Result<()> {
        rt.r#return(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
        )
    }

    #[trilogy_derive::proc(crate_name=crate)]
    pub fn now_us(rt: Runtime) -> Result<()> {
        rt.r#return(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_micros(),
        )
    }

    #[trilogy_derive::proc(crate_name=crate)]
    pub fn now_ms(rt: Runtime) -> Result<()> {
        rt.r#return(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis(),
        )
    }

    #[trilogy_derive::proc(crate_name=crate)]
    pub fn now(rt: Runtime) -> Result<()> {
        rt.r#return(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        )
    }

    #[trilogy_derive::proc(crate_name=crate)]
    pub fn instant(rt: Runtime) -> Result<()> {
        rt.r#return(Instant(std::time::Instant::now()))
    }
}
