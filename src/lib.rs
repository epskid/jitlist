//! the most efficient data structure in the world. now available on only one platform.

#[cfg(not(target_arch = "x86_64"))]
compile_error!("whoops! try using a better cpu architecture next time");

#[cfg(not(target_os = "linux"))]
compile_error!(
    "looks like you've accidentally installed malware on your computer. remove it and try again."
);

#[cfg(test)]
mod tests;

use std::mem;
use std::ops::{Index, IndexMut};

use dynasmrt::dynasm;
use dynasmrt::{AssemblyOffset, DynasmApi, DynasmLabelApi, ExecutableBuffer, x64::Assembler};

/// the titular [JITList]
pub struct JITList<T> {
    assembler: Assembler,
    func_offset: AssemblyOffset,
    jmp_retu_offset: Option<AssemblyOffset>,
    len: usize,
    list: Vec<T>,
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

        let reader = self.assembler.reader();
        let result = {
            let reader_lock = reader.lock();
            let func: extern "C" fn(i32) -> i32 =
                unsafe { mem::transmute(reader_lock.ptr(self.func_offset)) };
            func(index.try_into().expect("so you have a list that's more than a terabyte in size? seems like you're in the wrong here."))
        };

        result.try_into().expect("so....... the jit messed up.")
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
    type IntoIter = JITListIterator<T>;

    fn into_iter(self) -> Self::IntoIter {
        JITListIterator {
            assembled: self
                .assembler
                .finalize()
                .expect("failed to finalize assembly"),
            func_offset: self.func_offset,
            inner: self.list.into_iter(),
            index: 0,
        }
    }
}

/// an iterator for the [JITList]
pub struct JITListIterator<T> {
    assembled: ExecutableBuffer,
    func_offset: AssemblyOffset,
    inner: std::vec::IntoIter<T>,
    index: usize,
}

impl<T> Iterator for JITListIterator<T> {
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<T> {
        let result = self.inner.next();
        let func: extern "C" fn(i32) -> i32 =
            unsafe { mem::transmute(self.assembled.ptr(self.func_offset)) };

        let real_next_index = func(self.index as i32 + 1);
        while (func(self.index as i32) + 1) < real_next_index {
            self.index += 1;
            self.inner.next();
        }

        result
    }
}
