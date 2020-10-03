use crate::inner_prelude::*;

///Shorthand constructor of `BBox`
pub fn bbox<N, T>(rect: Rec<N>, inner: T) -> BBox<N, T> {
    BBox::new(rect, inner)
}

///A bounding box container object that implements Aabb and HasInner.
///Note that `&mut BBox<N,T>` also implements Aabb and HasInner.
///
///Using this one struct the user can construct the following types for bboxes to be inserted into the dinotree:
///
///* `BBox<N,T>`  (direct)
///* `&mut BBox<N,T>` (indirect)
///* `BBox<N,&mut T>` (rect direct, T indirect) (best performnace)
///* `BBox<N,*mut T>` (used internally by `DinoTreeOwned`)
///
///
///
///
#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct BBox<N, T> {
    pub rect: Rec<N>,
    pub inner: T,
}

impl<N, T> BBox<N, T> {
    #[inline(always)]
    pub fn new(rect: Rec<N>, inner: T) -> BBox<N, T> {
        BBox { rect, inner }
    }
}

unsafe impl<N: Num, T> Aabb for &mut BBox<N, T> {
    type Num = N;
    #[inline(always)]
    fn get_rec(&self) -> &Rec<Self::Num> {
        &self.rect
    }
}

unsafe impl<N: Num, T> HasInner for &mut BBox<N, T> {
    type Inner = T;

    #[inline(always)]
    fn get_inner(&self) -> (&Rect<N>, &Self::Inner) {
        (self.rect.get(), &self.inner)
    }

    #[inline(always)]
    fn get_inner_mut(&mut self) -> (&Rect<N>, &mut Self::Inner) {
        (self.rect.get(), &mut self.inner)
    }
}

unsafe impl<N: Num, T> Aabb for BBox<N, T> {
    type Num = N;
    #[inline(always)]
    fn get_rec(&self) -> &Rec<Self::Num> {
        &self.rect
    }
}

unsafe impl<N: Num, T> HasInner for BBox<N, T> {
    type Inner = T;

    #[inline(always)]
    fn get_inner(&self) -> (&Rect<N>, &Self::Inner) {
        (self.rect.get(), &self.inner)
    }

    #[inline(always)]
    fn get_inner_mut(&mut self) -> (&Rect<N>, &mut Self::Inner) {
        (self.rect.get(), &mut self.inner)
    }
}
