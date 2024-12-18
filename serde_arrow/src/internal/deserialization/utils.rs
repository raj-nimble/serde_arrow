use crate::internal::{
    arrow::BitsWithOffset,
    error::{fail, Result},
    utils::{array_ext::get_bit_buffer, Offset},
};

pub fn bitset_is_set(set: &BitsWithOffset<'_>, idx: usize) -> Result<bool> {
    get_bit_buffer(set.data, set.offset, idx)
}

pub struct ArrayBufferIterator<'a, T: Copy> {
    pub buffer: &'a [T],
    pub validity: Option<BitsWithOffset<'a>>,
    pub next: usize,
}

impl<'a, T: Copy> ArrayBufferIterator<'a, T> {
    pub fn new(buffer: &'a [T], validity: Option<BitsWithOffset<'a>>) -> Self {
        Self {
            buffer,
            validity,
            next: 0,
        }
    }

    pub fn next(&mut self) -> Result<Option<T>> {
        if self.next > self.buffer.len() {
            fail!("Exhausted deserializer");
        }

        if let Some(validity) = &self.validity {
            if !bitset_is_set(validity, self.next)? {
                return Ok(None);
            }
        }
        let val = self.buffer[self.next];
        self.next += 1;

        Ok(Some(val))
    }

    pub fn next_required(&mut self) -> Result<T> {
        let Some(next) = self.next()? else {
            fail!("Exhausted deserializer");
        };
        Ok(next)
    }

    pub fn peek_next(&self) -> Result<bool> {
        if self.next > self.buffer.len() {
            fail!("Exhausted deserializer");
        }

        if let Some(validity) = &self.validity {
            if !bitset_is_set(validity, self.next)? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    pub fn consume_next(&mut self) {
        self.next += 1;
    }
}

/// Check that the list layout given in terms of validity and offsets is
/// supported by serde_arrow
///
/// While the [arrow format spec][] explicitly allows null values in lists that
/// correspond to non-empty segments, this is currently not supported in arrow
/// deserialization. The spec says "a null value may correspond to a
/// **non-empty** segment in the child array."
///
/// [arrow format spec]: https://arrow.apache.org/docs/format/Columnar.html#variable-size-list-layout
pub fn check_supported_list_layout<'a, O: Offset>(
    validity: Option<BitsWithOffset<'a>>,
    offsets: &'a [O],
) -> Result<()> {
    if offsets.is_empty() {
        fail!("Unsupported: list offsets must be non empty");
    }

    for i in 0..offsets.len().saturating_sub(1) {
        let curr = offsets[i].try_into_usize()?;
        let next = offsets[i + 1].try_into_usize()?;

        if next < curr {
            fail!("Unsupported: list offsets are assumed to be monotonically increasing");
        }
        if let Some(validity) = validity.as_ref() {
            if !bitset_is_set(validity, i)? && (next - curr) != 0 {
                fail!("Unsupported: lists with data in null values are currently not supported in deserialization");
            }
        }
    }

    Ok(())
}

#[test]
fn test_check_supported_list_layout() {
    use crate::internal::testing::assert_error_contains;

    assert_error_contains(&check_supported_list_layout::<i32>(None, &[]), "non empty");
    assert_error_contains(
        &check_supported_list_layout::<i32>(None, &[0, 1, 0]),
        "monotonically increasing",
    );
    assert_error_contains(
        &check_supported_list_layout::<i32>(
            Some(BitsWithOffset {
                offset: 0,
                data: &[0b_101],
            }),
            &[0, 5, 10, 15],
        ),
        "data in null values",
    );
}
