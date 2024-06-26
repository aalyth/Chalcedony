use crate::utils::PtrString;

/// The bytecode instructions, generated by the `codegen` of the interpeter and
/// executed by the `CVM`.
#[derive(Debug, Clone, PartialEq)]
pub enum Bytecode {
    /// Does nothing.
    Nop,

    /// Copies the top of the stack.
    Dup,
    /// Pops the object at the top of the stack and pushes a new deeply copied
    /// version of it back.
    Copy,
    /// Removes the top of the stack.
    Pop,

    /// Pushes an `CvmObject::Int()` on the top of the stack.
    ConstI(i64),
    /// Pushes `CvmObject::Uint()` on the top of the stack.
    ConstU(u64),
    /// Pushes `CvmObject::Float()` on the top of the stack.
    ConstF(f64),
    /// Pushes `CvmObject::Str()` on the top of the stack.
    ConstS(PtrString),
    /// Pushes `CvmObject::Bool()` on the top of the stack.
    ConstB(bool),
    /// Pops the top N elements off the stack and builds an Object out of them.
    ConstObj(usize),
    /// Pops the top N elements off the stack and builds a list out of them.
    ConstL(usize),
    /// Converts the top of the stack from a `CvmObject::Str()` into a
    /// `CvmObject::Exception()`.
    ThrowException,

    /// Converts the top of the stack to a `CvmObjec::Int()`. Used to impicitly
    /// convert types of `Uint` to `Int` and for the builtins `utoi()`,
    /// `ftoi()`.
    CastI,

    /// Converts the top of the stack to a `CvmObjec::Float()`. Used to
    /// impicitly convert types of `Uint` and `Int` to `Float` and for the
    /// builtins `itof()`, `utof()`.
    CastF,

    // Converts the top of the stack to a `CvmObject::Uint()`. Used for the
    // builtins `itou()` and `ftou()`.
    CastU,

    /// Pops the top 2 operators off the stack and performs the corresponding
    /// binary operation. The result is then pushed back on the stack.
    Add,
    Sub,
    Mul,
    Div,
    Mod,

    /// Pops the top 2 operators off the stack and performs the corresponding
    /// logical operation. A value of type `CvmObject::Bool()` is pushed to the
    /// top of the stack.
    And,
    Or,
    Lt,
    Gt,
    Eq,
    LtEq,
    GtEq,

    /// Pops the top element off the stack, performs the corresponding unary
    /// operation and pushes back the resulting value on the stack.
    Neg,
    Not,

    /// Setters and getters for variables. Setting the value pops the top off
    /// the stack and replaces the value at the index. Getting the value pushes
    /// the corresponding variable's value onto the stack.

    /// Sets/gets variables from the CVM's globals hashmap.
    SetGlobal(usize),
    GetGlobal(usize),
    /// Sets/gets variables at the position on the stack.
    SetLocal(usize),
    GetLocal(usize),
    /// Sets/gets the attribute with `id` at from the top of the stack.
    SetAttr(usize),
    GetAttr(usize),

    /// Creates a new function, whose body is the remaining of the passed
    /// bytecode instructions. The argument describes the number of arguments
    /// the function has.
    CreateFunc(usize),
    /// Calls the function with the given id. The function's arguments must
    /// already be present at the top N positions on the stack.
    CallFunc(usize),

    /// Both operations remove the top call frame from the CVM call stack and
    /// truncate the remaing stack length to `call_frame.stack_length`. The
    /// `Return` operation keeps the returned value on the top of the stack.
    Return,
    ReturnVoid,

    /// If the top of the stack is `CvmObject::Bool(false)` jumps forward `N`
    /// instructions, else does nothing.
    If(usize),
    /// Moves the instruction counter forward or backwards `N` instructions.
    Jmp(isize),

    /// Defines the next `N` instructions as guarded within a `try-catch` scope.
    TryScope(usize),
    /// Used at the end of the `try` block to jump over the `catch` block.
    CatchJmp(usize),

    /// Pops the top element off the stack and pushes back it's length. Used
    /// for lists and strings.
    Len,
    /// Pops the `CvmObject::Int()` at the top off the stack and retrieves the
    /// element at the index of the list at the top of the stack.
    ListGet,
    /// Pops the top off the stack, interprets it as a list index, and pops the
    /// corresponding index off the list at the top of the stack. The removed
    /// value is pushed on the stack. If the index is invalid an exception is
    /// thrown.
    ListRemove,
    /// Pops the top off the stack, interprets it as a list index, pops the next
    /// element as the value to be inserted, and inserts it in the list at the
    /// top of the stack (`<list> <element> <idx>`). If the index is invalid an
    /// exception is thrown.
    ListInsert,
    /// Pops the top off the stack, interprets it as a list index, pops the next
    /// element as the value to be replaced, and sets it in the list at the top
    /// of the stack (`<list> <element> <idx>`). If the index is invalid an
    /// exception is thrown.
    ListSet,

    /// Pops the top value off the stack and outputs it to `stdout`.
    Print,
    /// Asserts the top of the stack is true, else an exception is thrown.
    Assert,
}
