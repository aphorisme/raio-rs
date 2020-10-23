#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Version {
    pub min: u8,
    pub maj: u8,
}

impl Version {
    /// Creates a new `Version` with provided major and minor increment.
    pub fn new(maj: u8, min: u8) -> Self {
        Version {
            maj,
            min,
        }
    }

    /// Creates a `Version` with no information.
    pub fn empty() -> Self {
        Version::new(0, 0)
    }

    /// Encodes `Version` as needed for the bolt protocol handshake. This packs minor and major in the
    /// last two bytes and leaves the first two bytes as 0:
    /// ```
    /// # use raio::ll::version::Version;
    /// assert_eq!([0, 0, 1, 4], Version::new(4, 1).encode());
    /// ```
    pub fn encode(&self) -> [u8; 4] {
        [0, 0, self.min, self.maj]
    }

    /// The inverse to `encode`, reads out 4 bytes into a version:
    /// ```
    /// # use raio::ll::version::Version;
    /// let bytes = [0, 0, 0, 3];
    /// let version = Version::decode(&bytes);
    ///
    /// assert_eq!(Version::new(3, 0), version);
    ///
    /// // is meant to be the inverse of encode:
    /// let version = Version::new(3, 5);
    /// assert_eq!(&version, &Version::decode(&version.encode()));
    /// ```
    pub fn decode(bytes: &[u8; 4]) -> Self {
        Version {
            maj: bytes[3],
            min: bytes[2]
        }
    }
}