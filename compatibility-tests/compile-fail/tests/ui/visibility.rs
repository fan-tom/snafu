// There are also happy-path tests

mod outer {
    pub mod inner {
        use snafu::Snafu;

        #[derive(Debug, Snafu)]
        #[snafu(visibility = "pub(crate)")]
        pub(crate) enum Error {
            PubCrate,
            #[snafu(visibility = "pub(in crate::outer)")]
            PubInPath,
            #[snafu(visibility)]
            Private,
        }
    }

    fn private_is_applied() {
        let _ = self::inner::Private.fail::<()>();
    }
}

fn pub_in_path_is_applied() {
    let _ = self::outer::inner::PubInPath.fail::<()>();
}

fn private_is_applied() {
    let _ = self::outer::inner::Private.fail::<()>();
}

fn main() {}
