/// A helpful util that works around lifetime issues
#[inline]
pub fn find_mut_or_push<'a, T, P, C>(vec: &'a mut Vec<T>, predicate: P, constructor: C) -> &'a mut T
where
    P: FnMut(&T) -> bool,
    C: FnOnce() -> T,
{
    if let Some(index) = vec.iter().position(predicate) {
        // SAFETY: The index is known to exist
        return unsafe { vec.get_unchecked_mut(index) };
    }

    vec.push(constructor());
    vec.last_mut().unwrap()
}
