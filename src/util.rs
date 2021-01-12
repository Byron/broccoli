use crate::inner_prelude::*;


pub fn combine_slice<'a, T>(a: &'a [T], b: &'a [T]) -> &'a [T] {
    let alen = a.len();
    let blen = b.len();
    unsafe {
        assert_eq!(
            a.as_ptr().offset(a.len() as isize),
            b.as_ptr(),
            "Slices are not continuous"
        );

        
        core::slice::from_raw_parts(a.as_ptr(), alen + blen)
    }
}

#[inline(always)]
pub fn compare_bots<T: Aabb>(axis: impl Axis, a: &T, b: &T) -> core::cmp::Ordering {
    let (p1, p2) = (a.get().get_range(axis).start, b.get().get_range(axis).start);
    if p1 > p2 {
        core::cmp::Ordering::Greater
    } else {
        core::cmp::Ordering::Less
    }
}

///Sorts the bots based on an axis.
#[inline(always)]
pub fn sweeper_update<I: Aabb, A: Axis>(axis: A, collision_botids: &mut [I]) {
    let sclosure = |a: &I, b: &I| -> core::cmp::Ordering { compare_bots(axis, a, b) };

    collision_botids.sort_unstable_by(sclosure);
}


pub use self::prevec::PreVec;
mod prevec {
    use crate::pmut::PMut;
    use twounordered::TwoUnorderedVecs;

    //The data in prevec is cleared before a vec is returned to the user
    unsafe impl<T:Send> core::marker::Send for PreVec<T> {}
    unsafe impl<T:Sync> core::marker::Sync for PreVec<T> {}

    ///An vec api to avoid excessive dynamic allocation by reusing a Vec
    pub struct PreVec<T> {
        vec: TwoUnorderedVecs<*mut T>,
    }

    impl<T> PreVec<T> {
        #[allow(dead_code)]
        #[inline(always)]
        pub fn new()->PreVec<T>{
            PreVec{
                vec:TwoUnorderedVecs::new()
            }
        }
        #[inline(always)]
        pub fn with_capacity(num: usize) -> PreVec<T> {
            PreVec {
                vec: TwoUnorderedVecs::with_capacity(num),
            }
        }

        ///Clears the vec and returns a mutable reference to a vec.
        #[inline(always)]
        pub fn get_empty_vec_mut<'a, 'b: 'a>(
            &'a mut self,
        ) -> &'a mut TwoUnorderedVecs<PMut<'b, T>> {
            self.vec.clear();
            let v: &mut TwoUnorderedVecs<_> = &mut self.vec;
            unsafe { &mut *(v as *mut _ as *mut TwoUnorderedVecs<_>) }
        }
    }
}

pub use self::slicesplit::SliceSplitMut;
mod slicesplit {
    use itertools::Itertools;

    ///Splits a mutable slice into multiple slices
    ///The splits occur where the predicate returns false.
    pub struct SliceSplitMut<'a, T, F> {
        arr: Option<&'a mut [T]>,
        func: F,
    }

    impl<'a, T, F: FnMut(&T, &T) -> bool> SliceSplitMut<'a, T, F> {
        pub fn new(arr: &'a mut [T], func: F) -> SliceSplitMut<'a, T, F> {
            SliceSplitMut {
                arr: Some(arr),
                func,
            }
        }
    }

    impl<'a, T, F: FnMut(&T, &T) -> bool> DoubleEndedIterator for SliceSplitMut<'a, T, F> {
        fn next_back(&mut self) -> Option<Self::Item> {
            let (last, arr) = {
                let arr = self.arr.take()?;
                let ll = arr.len();
                let i = arr.last()?;
                let count = arr
                    .iter()
                    .rev()
                    .peeking_take_while(|a| (self.func)(a, i))
                    .count();
                (ll - count, arr)
            };
            let (rest, last) = arr.split_at_mut(last);
            self.arr = Some(rest);
            Some(last)
        }
    }
    impl<'a, T, F: FnMut(&T, &T) -> bool> Iterator for SliceSplitMut<'a, T, F> {
        type Item = &'a mut [T];
        fn next(&mut self) -> Option<Self::Item> {
            let (last, arr) = {
                let arr = self.arr.take()?;
                let i = arr.get(0)?;
                let count = arr.iter().peeking_take_while(|a| (self.func)(a, i)).count();
                (count, arr)
            };
            let (first, rest) = arr.split_at_mut(last);
            self.arr = Some(rest);
            Some(first)
        }
    }
}
