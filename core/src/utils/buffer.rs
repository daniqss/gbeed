#[cfg(feature = "alloc")]
use alloc::{boxed::Box, vec};

/// Buffer to store the different memories of the Game Boy.
/// This allow us to easily switch from stack to heap and viceversa, just with a compile flag  
pub struct Buffer<const N: usize> {
    #[cfg(feature = "alloc")]
    data: Box<[u8]>,
    #[cfg(not(feature = "alloc"))]
    data: [u8; N],
    #[cfg(not(feature = "alloc"))]
    len: usize,
}

impl<const N: usize> Buffer<N> {
    pub fn zeroed(len: usize) -> Self {
        #[cfg(feature = "alloc")]
        {
            Self {
                data: vec![0u8; len].into_boxed_slice(),
            }
        }

        #[cfg(not(feature = "alloc"))]
        {
            assert!(len <= N, "buffer capacity exceeded: {len} > {N}");
            Self { data: [0u8; N], len }
        }
    }

    pub fn from_slice(slice: &[u8]) -> Self {
        #[cfg(feature = "alloc")]
        {
            Self {
                data: slice.to_vec().into_boxed_slice(),
            }
        }
        #[cfg(not(feature = "alloc"))]
        {
            assert!(
                slice.len() <= N,
                "buffer capacity exceeded: {} > {N}",
                slice.len()
            );
            let mut data = [0u8; N];
            data[..slice.len()].copy_from_slice(slice);
            Self {
                data,
                len: slice.len(),
            }
        }
    }

    pub fn as_slice(&self) -> &[u8] {
        #[cfg(feature = "alloc")]
        {
            &self.data
        }
        #[cfg(not(feature = "alloc"))]
        {
            &self.data[..self.len]
        }
    }

    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        #[cfg(feature = "alloc")]
        {
            &mut self.data
        }
        #[cfg(not(feature = "alloc"))]
        {
            &mut self.data[..self.len]
        }
    }
}

impl<const N: usize> Default for Buffer<N> {
    fn default() -> Self {
        #[cfg(feature = "alloc")]
        {
            Self { data: Box::default() }
        }
        #[cfg(not(feature = "alloc"))]
        {
            Self {
                data: [0u8; N],
                len: 0,
            }
        }
    }
}

impl<const N: usize> core::fmt::Debug for Buffer<N> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Buffer")
            .field("len", &self.as_slice().len())
            .finish()
    }
}

impl<const N: usize> core::ops::Deref for Buffer<N> {
    type Target = [u8];
    fn deref(&self) -> &[u8] { self.as_slice() }
}

impl<const N: usize> core::ops::DerefMut for Buffer<N> {
    fn deref_mut(&mut self) -> &mut [u8] { self.as_mut_slice() }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn zeroed_has_requested_length() {
        let buf: Buffer<16> = Buffer::zeroed(10);
        assert_eq!(buf.len(), 10);
        assert!(buf.iter().all(|&b| b == 0));
    }

    #[test]
    fn from_slice_copies_content() {
        let buf: Buffer<8> = Buffer::from_slice(&[1, 2, 3]);
        assert_eq!(&*buf, &[1, 2, 3]);
    }

    #[test]
    fn deref_mut_allows_writes() {
        let mut buf: Buffer<4> = Buffer::zeroed(4);
        buf[1] = 0xAB;
        assert_eq!(buf[1], 0xAB);
    }
}
