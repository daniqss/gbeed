use crate::cpu::Instruction;
use std::marker::Unsize;
use std::ops::{CoerceUnsized, Deref, DerefMut};
use std::ptr::NonNull;

/// # Instruction Box
/// Stack based box to store the DMG instructions (all instructions are 4 bytes or less).
/// This allow us to store the instructions without heap allocation, and still have dynamic dispatch for the instructions.
pub struct InstructionBox<T: ?Sized> {
    data: [u32; 1],
    metadata: NonNull<T>,
}

// CoerceUnsized allows us to coerce InstructionBox<T> to InstructionBox<U>, so we can:
/// ``` rust
/// let instruction: InstructionBox<dyn Instruction> = match opcode {
///     // no need to transmute from InstructionBox<Nop> to InstructionBox<dyn Instruction>
///     0x00 => InstructionBox::new(Nop),
///     ...
/// }
///
/// impl Nop {
///     new() -> InstructionBox<Self> { InstructionBox::new(Self) }
/// }
///
/// impl Instruction for Nop {
///     fn exec(&mut self, _: &mut Dmg) -> InstructionResult {
///         Ok(InstructionEffect::new(self.info(), Flags::none()))
///     }
///     fn info(&self) -> (u8, u8) { (1, 1) }
///     fn disassembly(&self) -> String { "nop".to_string() }
/// }
/// ```
///
/// This way, the linter doesn't complain about new not returning `Self`.
impl<T, U> CoerceUnsized<InstructionBox<U>> for InstructionBox<T>
where
    T: ?Sized + Unsize<U>,
    U: ?Sized,
{
}

impl<T: Instruction + Copy + 'static> InstructionBox<T> {
    pub fn new(val: T) -> Self {
        const {
            assert!(std::mem::size_of::<T>() <= 4, "Instruction too big");
            assert!(std::mem::align_of::<T>() <= 4, "Alignment too large");
        }

        let mut data = [0u32; 1];
        unsafe {
            std::ptr::write(data.as_mut_ptr() as *mut T, val);
        }

        InstructionBox {
            data,
            metadata: NonNull::dangling(),
        }
    }
}

impl<T: ?Sized + Instruction> Deref for InstructionBox<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        // SAFETY: we need to reconstruct the instruction using the raw data and the vtable from the metadata
        unsafe {
            let raw_data = self.data.as_ptr() as *const ();
            let metadata = std::ptr::metadata(self.metadata.as_ptr());

            &*std::ptr::from_raw_parts(raw_data, metadata)
        }
    }
}

impl<T: ?Sized + Instruction> DerefMut for InstructionBox<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: we need to reconstruct the instruction using the raw data and the vtable from the metadata
        unsafe {
            let raw_data = self.data.as_mut_ptr() as *mut ();
            let metadata = std::ptr::metadata(self.metadata.as_ptr());

            &mut *std::ptr::from_raw_parts_mut(raw_data, metadata)
        }
    }
}
