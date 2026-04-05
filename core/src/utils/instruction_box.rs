use crate::cpu::Instruction;

pub struct InstructionBox {
    data: [u32; 1],
    vtable: *const (),
}

impl InstructionBox {
    pub fn new<T: Instruction + Copy + 'static>(val: T) -> Self {
        const {
            assert!(
                std::mem::size_of::<T>() <= 4,
                "Instruction too big to fit in the box"
            )
        }
        const {
            assert!(
                std::mem::align_of::<T>() <= 4,
                "Instruction with alignment too large to fit in the box"
            )
        }

        // we get the vtable pointer for the trait object by transmuting a reference to the value as a trait object reference
        // the first element is the data pointer which contains the value of the instruction
        // but we will not use it directly since we will store the value in the data array of the InstructionBox
        // the second element is the vtable pointer which contains the function pointers for the trait methods
        let vtable = unsafe {
            let (_data_ptr, vtable_ptr): (*const (), *const ()) =
                std::mem::transmute::<&dyn Instruction, _>(&val as &dyn Instruction);
            vtable_ptr
        };

        let mut data = [0u32; 1];
        unsafe { std::ptr::write(data.as_mut_ptr() as *mut T, val) };

        InstructionBox { data, vtable }
    }

    #[inline]
    fn as_dyn_ptr(&self) -> *const dyn Instruction {
        unsafe { std::mem::transmute((self.data.as_ptr() as *const (), self.vtable)) }
    }

    #[inline]
    fn as_dyn_ptr_mut(&mut self) -> *mut dyn Instruction {
        unsafe { std::mem::transmute((self.data.as_mut_ptr() as *const (), self.vtable)) }
    }
}

impl std::ops::Deref for InstructionBox {
    type Target = dyn Instruction;
    #[inline]
    fn deref(&self) -> &Self::Target { unsafe { &*self.as_dyn_ptr() } }
}

impl std::ops::DerefMut for InstructionBox {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target { unsafe { &mut *self.as_dyn_ptr_mut() } }
}

impl std::fmt::Display for InstructionBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { std::fmt::Display::fmt(&**self, f) }
}

impl std::fmt::Debug for InstructionBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{}", self.disassembly()) }
}
