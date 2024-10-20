use std::collections::HashSet;

use crate::vm::VM;

pub struct Dbg<'a> {
    pub breakpoints: HashSet<u32>,
    pub vm: VM<'a>
}
impl <'a> Dbg <'a> {
    #[inline]
    pub fn new(vm: VM<'a>) -> Self {
        Self { vm, breakpoints: HashSet::new() }
    }

    pub fn next(&mut self) {
        self.vm.next();
    }
    pub fn r#continue(&mut self) {
        loop {
            if self.breakpoints.contains(&(self.vm.ip as u32)) {
                return;
            }
            self.next();
        }
    }
    pub fn disasm(&mut self) {
        eprint!("{:08X}>",self.vm.ip);
        self.vm.disasm(self.vm.ip());
    }
}
