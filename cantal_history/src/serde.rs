probor_struct!(
#[derive(PartialEq, Eq, Debug)]
pub struct VersionInfo {
    version: u8 => (),
});

impl VersionInfo {
    pub fn current() -> VersionInfo {
        VersionInfo { version: 2 }
    }
}

