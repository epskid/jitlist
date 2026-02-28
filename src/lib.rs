#![feature(impl_trait_in_assoc_type)]

//! the most efficient data structure in the world. now available on only one platform.

#[cfg(not(target_arch = "x86_64"))]
compile_error!("whoops! try using a better cpu architecture next time");

#[cfg(not(target_os = "linux"))]
compile_error!(
    "looks like you've accidentally installed malware on your computer. remove it and try again."
);

#[cfg(test)]
mod tests;

use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::mem;
use std::ops::{Index, IndexMut};

use dynasmrt::dynasm;
use dynasmrt::{AssemblyOffset, DynasmApi, DynasmLabelApi, x64::Assembler};

/// the titular [JITList]
pub struct JITList<T> {
    assembler: Assembler,
    func_offset: AssemblyOffset,
    jmp_retu_offset: Option<AssemblyOffset>,
    len: usize,
    list: Vec<T>,
    removed: BinaryHeap<Reverse<usize>>,
}

impl<T> JITList<T> {
    /// fallibly create a [JITList] from a [Vec]
    pub fn try_new(list: Vec<T>) -> anyhow::Result<Self> {
        let mut assembler = Assembler::new_with_capacity(2 * list.len())?;

        dynasm!(assembler
            ; .arch x64
            ; ->retu:
            ; pop rbp
            ; ret
        );

        let func_offset = assembler.offset();
        dynasm!(assembler
            ; .arch x64
            ; push rbp
            ; mov rbp, rsp
            ; mov DWORD [rbp-4], edi
            ; mov eax, DWORD [rbp-4]
        );

        Ok(Self {
            assembler,
            func_offset,
            jmp_retu_offset: None,
            len: list.len(),
            list,
            removed: BinaryHeap::new(),
        })
    }

    /// calls [JITList::try_new] and assumes your computer can handle the power.
    pub fn new(list: Vec<T>) -> Self {
        Self::try_new(list).expect("you couldn't handle the power")
    }

    /// retrieve the length of a [JITList]
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.len
    }

    /// check if a [JITList] is empty
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[inline(always)]
    fn check_index(&self, index: usize) {
        if index >= self.len {
            panic!("index {index} out of bounds for length {}", self.len);
        }
    }

    /// fallibly remove an element from the [JITList]
    #[inline]
    pub fn try_remove(&mut self, index64: usize) -> anyhow::Result<()> {
        self.check_index(index64);

        let index: i32 = index64.try_into()?;

        if !self.removed.is_empty() {
            let reader = self.assembler.reader();
            let reader_lock = reader.lock();
            let func: extern "C" fn(i32) -> i32 =
                unsafe { mem::transmute(reader_lock.ptr(self.func_offset)) };

            self.removed.push(Reverse(func(index) as usize));
        } else {
            self.removed.push(Reverse(index64));
        }

        self.len -= 1;

        // overwrite previous jump with a nop jump
        // i couldn't find a way to remove instructions
        // and nop isn't aligned properly
        if let Some(jmp_retu_offset) = self.jmp_retu_offset.take() {
            let nop_label = self.assembler.new_dynamic_label();

            self.assembler.alter(|alterer| {
                alterer.goto(jmp_retu_offset);

                dynasm!(alterer
                    ; .arch x64
                    ; jmp =>nop_label
                    ; =>nop_label
                );
            })?;
        }

        let not_affected = self.assembler.new_dynamic_label();

        dynasm!(self.assembler
            ; .arch x64
            ; cmp DWORD [rbp - 4], DWORD index
            ; jl =>not_affected
            ; add eax, 1
            ; =>not_affected
        );

        self.jmp_retu_offset = Some(self.assembler.offset());

        dynasm!(self.assembler
            ; .arch x64
            ; jmp ->retu
        );

        self.assembler.commit()?;

        Ok(())
    }

    /// powerfully assert your dominance over the puny machine. calls [JITList::try_remove] and
    /// explode if it fails.
    pub fn remove(&mut self, index: usize) {
        self.try_remove(index).expect("you have been bested");
    }

    #[inline]
    fn get_real_index(&self, index: usize) -> usize {
        self.check_index(index);

        if !self.removed.is_empty() {
            let reader = self.assembler.reader();
            let result = {
                let reader_lock = reader.lock();
                let func: extern "C" fn(i32) -> i32 =
                    unsafe { mem::transmute(reader_lock.ptr(self.func_offset)) };
                func(index.try_into().expect("so you have a list that's more than a terabyte in size? seems like you're in the wrong here."))
            };

            result.try_into().expect("so....... the jit messed up.")
        } else {
            index
        }
    }
}

impl<T> Index<usize> for JITList<T> {
    type Output = T;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        self.list.index(self.get_real_index(index))
    }
}

impl<T> IndexMut<usize> for JITList<T> {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.list.index_mut(self.get_real_index(index))
    }
}

impl<T> IntoIterator for JITList<T> {
    type Item = T;
    type IntoIter = impl Iterator<Item = Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        let mut removed = self.removed;
        self.list.into_iter().enumerate().filter_map(move |(i, x)| {
            if removed.peek().is_some_and(|j| j.0 == i) {
                removed.pop();
                None
            } else {
                Some(x)
            }
        })
    }
}

impl<'a, T> IntoIterator for &'a JITList<T> {
    type Item = &'a T;
    type IntoIter = impl Iterator<Item = Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        let mut removed = self.removed.clone();
        self.list.iter().enumerate().filter_map(move |(i, x)| {
            if removed.peek().is_some_and(|j| j.0 == i) {
                removed.pop();
                None
            } else {
                Some(x)
            }
        })
    }
}

impl<'a, T> IntoIterator for &'a mut JITList<T> {
    type Item = &'a mut T;
    type IntoIter = impl Iterator<Item = Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        let mut removed = self.removed.clone();
        self.list.iter_mut().enumerate().filter_map(move |(i, x)| {
            if removed.peek().is_some_and(|j| j.0 == i) {
                removed.pop();
                None
            } else {
                Some(x)
            }
        })
    }
}
