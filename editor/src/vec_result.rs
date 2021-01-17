use crate::error::EdResult;
use crate::error::OutOfBounds;
use snafu::OptionExt;
use std::slice::SliceIndex;

// replace vec methods that return Option with ones that return Result and proper Error

pub fn get_res<T>(index: usize, slice: &[T]) -> EdResult<&<usize as SliceIndex<[T]>>::Output> {
    let elt_ref = slice.get(index).context(OutOfBounds {
        index,
        collection_name: "Slice",
        len: slice.len(),
    })?;

    Ok(elt_ref)
}
