/// Label for use as an instruction index
pub type Label = u32;

/// Local variable label / ID
pub type VariableLabel = u16;

#[derive(Debug, displaydoc::Display)]
#[ignore_extra_doc_attributes]
#[repr(u8)]
pub enum Instruction {
    /// No Operation
    Nop,

    /// Push a constant onto the stack
    Constant(Label),

    /// Pops a value from the stack
    Pop,

    /// Duplicates the top value of the stack
    Dup,

    /// Pushes a variables contents on the stack
    Load(VariableLabel),

    /// Pops a value and stores in the variable
    Store(VariableLabel),

    /// Jumps stackframe to the given function
    Call(Label),

    /// Pops a value and calls the function (object or raw pointer)
    DynCall,

    /// Return from this function
    Return,

    /// Jump to a given location
    Jump(Label),

    /// Jump to a given location if the popped value on the stack is false
    JumpIfFalse(Label),

    /// Pops a value and pushes it after using the not operation (!)
    Not,

    /// Pops 2 values and pushes the Or (|) result
    Or,

    /// Pops 2 values and pushes the And (&&) result
    And,

    /// Pops 2 values and pushes the equality comparison result (==)
    Equals,

    /// Pops 2 values and pushes the comparison result (>)
    GreaterThan,

    /// Pops 2 values and pushes the comparison result (<)
    LessThan,

    /// Pops 2 values and pushes the result (&)
    BitAnd,

    /// Pops 2 values and pushes the result (|)
    BitOr,

    /// Pops 2 values and pushes the result (^)
    BitXor,

    /// Pops 2 values and pushes the result (<<)
    BitShiftLeft,

    /// Pops 2 values and pushes the result (>>)
    BitShiftRight,

    /// Pops 2 values and pushes the result (+)
    Add,

    /// Pops 2 values and pushes the result (-)
    Subtract,

    /// Pops 2 values and pushes the result (*)
    Multiply,

    /// Pops 2 values and pushes the result (/)
    Divide,

    /// Pops 2 values and pushes the result (%)
    Modulo,

    /// Pops 2 values and pushes the result (-)
    Negate,
}

impl OpCode {
    /// How much does this instruction effect the stack pointer
    fn stack_offset(&self) -> i32 {
        match self {
            OpCode::Nop => 0,

            OpCode::Constant(..) => 1,
            OpCode::Pop => -1,

            OpCode::Dup => 1,
            OpCode::Load(..) => 1,
            OpCode::Call(..) => 1,

            OpCode::DynCall => 0,

            OpCode::Store(..) => -1,

            OpCode::Return => 0,

            OpCode::Jump(..) | OpCode::JumpIfFalse(..) => 0,

            // Unary Operations
            OpCode::Not | OpCode::Negate => 0,

            // Binary Operations
            OpCode::Or
            | OpCode::And
            | OpCode::Equals
            | OpCode::GreaterThan
            | OpCode::LessThan
            | OpCode::BitAnd
            | OpCode::BitOr
            | OpCode::BitXor
            | OpCode::BitShiftLeft
            | OpCode::BitShiftRight
            | OpCode::Add
            | OpCode::Subtract
            | OpCode::Multiply
            | OpCode::Divide
            | OpCode::Modulo => -1,
        }
    }
}
