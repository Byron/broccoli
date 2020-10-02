//! ## Overview
//!
//! For most usecases, using broccoli::Tree is enough. But in certain cases
//! we want more control. The container trees in this module are for this purpose.
//!
//! For example, with the regular broccoli::Tree, is lifetimed, so
//! it can't act as a container. You  also can't do the the following.
//!
//! ```rust,ignore
//! use axgeom::*;
//! use broccoli::prelude::*;
//! let mut k=[bbox(rect(0,10,0,10),())];
//! let b=broccoli::new(&mut k);
//! b.find_intersections_mut(|a,b|{});
//! k[0]=bbox(rect(20,40,20,40),());    //<---cannot re-borrow
//! b.find_intersections_mut(|a,b|{});
//! ```
//! This is because broccoli::Tree constructs itself by splitting up the 
//! passed mutable slice to the point where the original mutable slice
//! can't be retrieved.
//! 
//!
//!
//! ## An owned `(Rect<N>,T)` example
//!
//! ```rust
//! use broccoli::{prelude::*,collections::*,DefaultA};
//! use axgeom::*;
//!
//! fn not_lifetimed()->TreeOwned<DefaultA,BBox<i32,f32>>
//! {
//!     let a=vec![bbox(rect(0,10,0,10),0.0)];
//!     TreeOwned::new(a)
//! }
//!
//! not_lifetimed();
//!
//! ```
//!
//! ## An owned `(Rect<N>,*mut T)` example
//!
//! ```rust
//! use broccoli::{*,collections::*,DefaultA};
//! use axgeom::*;
//!
//! fn not_lifetimed()->TreeOwnedInd<DefaultA,i32,Vec2<i32>>
//! {
//!     let rect=vec![vec2(0,10),vec2(3,30)];
//!     TreeOwnedInd::new(rect,|&p|{
//!         let radius=vec2(10,10);
//!         Rect::from_point(p,radius)
//!     })
//! }
//!
//! not_lifetimed();
//!
//! ```

use super::*;


struct ThreadPtr<T>(*mut T);
unsafe impl<T> Send for ThreadPtr<T>{}
unsafe impl<T> Sync for ThreadPtr<T>{}


pub struct TreeRefInd<'a,A:Axis,N:Num,T>{
    inner:TreeOwned<A,BBox<N,*mut T>>,
    orig:*mut [T],
    _p:PhantomData<&'a mut T>
}

impl<'a,N:Num,T> TreeRefInd<'a,DefaultA,N,T>{
    pub fn new(arr:&'a mut [T],func:impl FnMut(&mut T)->Rect<N>)->TreeRefInd<'a,DefaultA,N,T>{
        TreeRefInd::with_axis(default_axis(),arr,func)
    }
}

impl<'a,A:Axis,N:Num,T> TreeRefInd<'a,A,N,T>{
    pub fn with_axis(axis:A,arr:&'a mut [T],mut func:impl FnMut(&mut T)->Rect<N>)->TreeRefInd<'a,A,N,T>{
        let orig=arr as *mut _;
        let bbox = arr
        .iter_mut()
        .map(|b| BBox::new(func(b), b as *mut _))
        .collect();

        let inner=TreeOwned::with_axis(axis,bbox);

        TreeRefInd{
            inner,
            orig,
            _p:PhantomData
        }
    }
    pub fn get_elements(&self)->&[T]{
        unsafe{&*self.orig}
    }
    pub fn get_elements_mut(&mut self)->&'a mut [T]{
        unsafe{&mut *self.orig}
    }
    pub fn get_tree_elements_mut(&mut self)->PMut<[BBox<N,&mut T>]>{
        //unsafe{&mut *(self.inner.get_elements_mut() as *mut _ as *mut _)}
        unimplemented!();

    }
    pub fn get_tree_elements(&self)->&[BBox<N,&mut T>]{
        unimplemented!();
    }
}


impl<'a,A:Axis,N:Num+'a,T> core::ops::Deref for TreeRefInd<'a,A,N,T>{
    type Target=Tree<'a,A,BBox<N,&'a mut T>>;
    fn deref(&self)->&Self::Target{
        //TODO do these in one place
        unsafe{&*(self.inner.as_tree() as *const _ as *const _)}
    }
}
pub struct TreeRef<'a,A:Axis,T:Aabb>{
    inner:TreeInner<A,NodePtr<T>>,
    orig:*mut [T],
    _p:PhantomData<&'a mut T>
}

impl<'a,A:Axis,T:Aabb> core::ops::Deref for TreeRef<'a,A,T>{
    type Target=Tree<'a,A,T>;
    fn deref(&self)->&Self::Target{
        //TODO do these in one place
        unsafe{&*(&self.inner as *const _ as *const _)}
    }
}
impl<'a,A:Axis,T:Aabb> core::ops::DerefMut for TreeRef<'a,A,T>{
    fn deref_mut(&mut self)->&mut Self::Target{
        //TODO do these in one place
        unsafe{&mut *(&mut self.inner as *mut _ as *mut _)}
    }
}

impl<'a,T:Aabb> TreeRef<'a,DefaultA,T>{
    pub fn new(arr:&'a mut [T])->TreeRef<'a,DefaultA,T>{
        TreeRef::with_axis(default_axis(),arr)
    }
}

impl<'a,A:Axis,T:Aabb> TreeRef<'a,A,T>{
    pub fn with_axis(a:A,arr:&'a mut [T])->TreeRef<'a,A,T>{
        let inner=make_owned(a,arr);
        let orig=arr as *mut _;
        TreeRef{
            inner,
            orig,
            _p:PhantomData
        }        
    }
    pub fn get_elements(&self)->&[T]{
        unsafe{&*self.orig}
    }
    pub fn get_elements_mut(&mut self)->PMut<'a,[T]>{
        PMut::new(unsafe{&mut *self.orig})
    }
}



///A Node in a Tree.
pub(crate) struct NodePtr<T: Aabb> {
    _range: PMutPtr<[T]>,

    //range is empty iff cont is none.
    _cont: Option<axgeom::Range<T::Num>>,
    //for non leafs:
    //  div is some iff mid is nonempty.
    //  div is none iff mid is empty.
    //for leafs:
    //  div is none
    _div: Option<T::Num>,
}

pub(crate) fn make_owned<A: Axis, T: Aabb>(axis: A, bots: &mut [T]) -> TreeInner<A, NodePtr<T>> {
    
    let inner = crate::with_axis(axis, bots);
    let inner: Vec<_> = inner
        .inner
        .inner
        .into_nodes()
        .drain(..)
        .map(|mut node| NodePtr {
            _range: node.range.as_ptr(),
            _cont: node.cont,
            _div: node.div,
        })
        .collect();
    let inner = compt::dfs_order::CompleteTreeContainer::from_preorder(inner).unwrap();
    TreeInner {
        axis,
        inner
    }
}

fn make_owned_par<A: Axis, T: Aabb + Send + Sync>(axis: A, bots: &mut [T]) -> TreeInner<A, NodePtr<T>> {
    let inner = crate::with_axis_par(axis, bots);
    let inner: Vec<_> = inner
        .inner
        .inner
        .into_nodes()
        .drain(..)
        .map(|mut node| NodePtr {
            _range: node.range.as_ptr(),
            _cont: node.cont,
            _div: node.div,
        })
        .collect();
    let inner = compt::dfs_order::CompleteTreeContainer::from_preorder(inner).unwrap();
    TreeInner {
        axis,
        inner
    }
}


pub struct TreeOwnedInd<A: Axis,N:Num, T> {
    inner:TreeOwned<A,BBox<N,ThreadPtr<T>>>,
    bots:Vec<T>
}

impl<N:Num,T:Send+Sync> TreeOwnedInd<DefaultA,N,T>{
    pub fn new_par(bots: Vec<T>,func:impl FnMut(&T)->Rect<N>) -> TreeOwnedInd<DefaultA,N, T> {
        TreeOwnedInd::with_axis_par(default_axis(),bots,func)
    }
}
impl<A:Axis,N:Num,T:Send+Sync> TreeOwnedInd<A,N,T>{
    pub fn with_axis_par(axis: A, mut bots: Vec<T>,mut func:impl FnMut(&T)->Rect<N>) -> TreeOwnedInd<A,N, T> {
        let bbox = bots
            .iter_mut()
            .map(|b| BBox::new(func(b), ThreadPtr(b as *mut _) ))
            .collect();
        
        let inner= TreeOwned::with_axis_par(axis,bbox); 
        TreeOwnedInd {
            inner,
            bots,
        }
        
    }
}

impl<N:Num,T> TreeOwnedInd<DefaultA,N, T> {
    pub fn new(bots: Vec<T>,func:impl FnMut(&T)->Rect<N>) -> TreeOwnedInd<DefaultA, N,T> {
        Self::with_axis(default_axis(), bots,func)
    }    
}
impl<A:Axis,N:Num,T> TreeOwnedInd<A,N,T>{
    ///Create an owned Tree in one thread.
    pub fn with_axis(axis: A, mut bots: Vec<T>,mut func:impl FnMut(&T)->Rect<N>) -> TreeOwnedInd<A,N, T> {
        let bbox = bots
            .iter_mut()
            .map(|b| BBox::new(func(b), ThreadPtr(b as *mut _)))
            .collect();
        
        let inner= TreeOwned::with_axis(axis,bbox); 
        TreeOwnedInd {
            inner,
            bots,
        }
        
    }

    ///Cant use Deref because of lifetime
    pub fn as_tree(&self)->&Tree<A,BBox<N,&mut T>>{
        unsafe{&*(self.inner.as_tree() as *const _ as *const _)}
    }

    ///Cant use Deref because of lifetime
    pub fn as_tree_mut(&mut self)->&mut Tree<A,BBox<N,&mut T>>{
        unsafe{&mut *(self.inner.as_tree_mut() as *mut _ as *mut _)}
    }

    
    pub fn get_elements(&self) -> &[T] {
        &self.bots
    }
    pub fn get_elements_mut(&mut self) -> &mut [T] {
        &mut self.bots
    }
}



///An owned Tree componsed of `T:Aabb`
pub struct TreeOwned<A: Axis, T: Aabb> {
    tree: TreeInner<A, NodePtr<T>>,
    bots: Vec<T>,
}

impl<T: Aabb+Send+Sync> TreeOwned<DefaultA, T> {
    pub fn new_par(bots:Vec<T>)->TreeOwned<DefaultA,T>{
        TreeOwned::with_axis_par(default_axis(),bots)
    }
}
impl<A: Axis, T: Aabb+Send+Sync> TreeOwned<A, T> {
    pub fn with_axis_par(axis:A,mut bots:Vec<T>)->TreeOwned<A,T>{
        TreeOwned{
            tree:make_owned_par(axis,&mut bots),
            bots
        }
    }
}
impl<T: Aabb> TreeOwned<DefaultA, T> {
    pub fn new(bots: Vec<T>) -> TreeOwned<DefaultA, T> {
        Self::with_axis(default_axis(), bots)
    }
    
}
impl<A: Axis, T: Aabb> TreeOwned<A, T> {
    ///Create an owned Tree in one thread.
    pub fn with_axis(axis: A, mut bots: Vec<T>) -> TreeOwned<A, T> {
        TreeOwned {
            tree: make_owned(axis, &mut bots),
            bots,
        }
    }
    
    ///Cant use Deref because of lifetime
    pub fn as_tree(&self)->&Tree<A,T>{
        unsafe{&*(&self.tree as *const _ as *const _)}
    }

    ///Cant use Deref because of lifetime
    pub fn as_tree_mut(&mut self)->&mut Tree<A,T>{
        unsafe{&mut *(&mut self.tree as *mut _ as *mut _)}
    }

    pub fn get_elements(&self) -> &[T] {
        &self.bots
    }
    pub fn get_elements_mut(&mut self) -> PMut<[T]> {
        PMut::new(&mut self.bots)
    }
    pub fn rebuild(&mut self, mut func: impl FnMut(&mut [T])) {
        func(&mut self.bots);

        let axis = self.tree.axis;
        self.tree = make_owned(axis, &mut self.bots);
    }

}
