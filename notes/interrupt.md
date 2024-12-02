# Charizard - Operating System written in Rust

## Part 2. Interrupts

### Step 1. CPU Exceptions

CPU exceptions occurs in various jerroneous situations, for example, when accessing an icalid memory address or whan dividing by zero. To react to them, we have to set up an `interrupt descriptor` talbe that provides handler functions.

An exception signals that something is wrongwith the current instruction. For example, the CPU issues an exception if the current instruction triestodivid by 0. When an exception occurs, the CPU interrupts its current work and immediately calls a specific exception handler function, depending on the exception type.

On x86, there are about 20 different CPU exception types. The most important are:

- **Page Fault**: A page fault occurs on illegal memory accesses.
- **Invalid Opcode** This exception occurs when the current instruction is invalid.
- **General Protection Fault**: This is the exeception with the broadest range of causes. It occurs on various kinds of access violations, such as trying to execute a privileged instruction in user-level code or writing reserved fields in configuration register.
- **Double Fault**: When an exception occurs, the CPU tries to call corresponding handler function. If another exception occurs while calling the exception handler, the CPU raises a double fault exeception. This exception also occurs when there is no handler function registered for an exception.
- **Triple Fault**: If an exception occurs while the CPU tries to call the double fault handler function, it issues a fatal triple fault. We can't catch or handle a triple fault. Most processors react by resetting themselves and rebooting the operating system.

#### The Interrupt Descriptor Table

To catch and handle exceptions, we have to set up a `Interrupt Descriptor Table(IDT)`. In this table, we can specify a handler function for each CPU exception. The hardware uses this table directly, so we need to follow a predefined format. Each entry must have the following `16 byte` structure.

#### Table 1: Structure Description

| Type | Name                     | Description                                                |
| ---- | ------------------------ | ---------------------------------------------------------- |
| u16  | Function Pointer [0:15]  | The lower bits of the pointer to the handler function.     |
| u16  | GDT selector             | Selector of a code segment in the global descriptor table. |
| u16  | Options                  | (see below)                                                |
| u16  | Function Pointer [16:31] | The middle bits of the pointer to the handler function.    |
| u32  | Function Pointer [32:63] | The remaining bits of the pointer to the handler function. |
| u32  | Reserved                 |                                                            |

#### Table 2: Options Field Format

| Bits  | Name                             | Description                                                                                                     |
| ----- | -------------------------------- | --------------------------------------------------------------------------------------------------------------- |
| 0-2   | Interrupt Stack Table Index      | 0: Don’t switch stacks, 1-7: Switch to the n-th stack in the Interrupt Stack Table when this handler is called. |
| 3-7   | Reserved                         |                                                                                                                 |
| 8     | 0: Interrupt Gate, 1: Trap Gate  | If this bit is 0, interrupts are disabled when this handler is called.                                          |
| 9-11  | must be one                      |                                                                                                                 |
| 12    | must be zero                     |                                                                                                                 |
| 13-14 | Descriptor Privilege Level (DPL) | The minimal privilege level required for calling this handler.                                                  |
| 15    | Present                          |                                                                                                                 |

Each exception has a prediefined IDT index. For example, the incalid opcode has table index 6 and the page fault exception has table index 14. Thus, the hardware can automatically load the corresponding IDT entry for each exception.

When an exception occurs, the CPU roughly does the following:

1. Push some registers on the stack, including the instruction pointer and the `RFLAGS` register.
2. Read the corresponding entry from the interrupt Descriptor Table. For example, the CPU reads the 14th entry whan page fault occurs.
3. Check if the entry is present and raise a double fault if not.
4. Disable harfware interrupts if the entry is an interrupt gate.
5. Load the specified GDT selector into the code segment.
6. Jump to the specifed handler function.

#### The Interrupt Calling Convention

Exceptions are quite similar to function calls: The CPU jumps to the first instruction of the called function and ececutes it. Afterwards, the CPU jumps to the return address and continues the execution of the parent function.

However, there is a major difference between exceptions and function calls: A function call is invoked voluntarily by a compiler-inserted `call` instruction, while an exception might occur at any instruction. In order to inderstand the consequences of this difference, we need to examine function calls in more detail.

Calling conventions specify the details of a function call. For example, they specify where function parameters are placed (e.g in registers or on the stack) and how results are returned. On x86_64 Linux, the following rules apply for C functions (specified in the Syetem V ABI):

1. The first 6 integer argumemts are passed in registers `rdi`, `rsi`, `rdx`, `rcx`, `r8`, `r9`
2. Additional arguments are passed on the stack
3. Results are returned in `rax` and `rdx`

##### Preserved and Scratch Registers

The calling convention divides the registers into two parts: `preserved` and `scratch` register.

The value of `preserved` registers must remain unchanged across function calls. So a called function is only allowed to overwrite these registers if it restores their original values before returning. Hence, these registers are called `callee-saved`. A common pattern is to save these registers to the stack at the function's befinning and restore them just before returning.

On the other hand, a called function is allowed to overwrite values in the `scratch` registers without restrications. If the caller wants to preserve the value of scratch register across a function call, it needs to backup and restore it before the function call (e.g. by pushing it to the stack). So the scratch registers are `caller-saved`

On x86_64, the C calling convention specifies the following preserved and scratch registers:

| Preserved Registers               | Scratch Registers                         |
| --------------------------------- | ----------------------------------------- |
| rbp, rbx, rsp, r12, r13, r14, r15 | rax, rcx, rdx, rsi, rdi, r8, r9, r10, r11 |
| callee-saved                      | caller-saved                              |

These complier knows these rules, so it generates the code accordingly. For example, most functions begin with a `push rdp`, which backups `rbp` on the stack.

##### Preserving all Registers

In contrast to function calls, exceptions can occur on any instruction. In most cases, we don't even know at compile time if the generated code will cause an exception. For example, the compiler won't know if an instruction causes a stack overflow or a page fault.

Since we don't know when an exception occurs, we can't backup any register before. This means we can't use a calling convention that relies on caller-saved registers for exception handlers. Instead, we need a calling convention that preserves `all registers`. The `x86-interrupt` calling convention is such a calling convention, so it guarantees that all register values are restored to their original values on function return.

However, to efficiently utilize registers, the compiler only backs up the registers that are overwritten by the function instead of saving all registers to the stack at function entry.

### The Interrupt Stack Frame

On a normal function call, the CPU pushes the return address before jumping to the target function. On the function return, the CPU pops this return address and jumps to it.

For exception and interrupt handlers, however, pushing a return address would not suffice, since interrupt handlers often run in a different context (stack pointer, CPU flags, etc.). Instead, the CPU performs the following steps when an interrupt occurs:

1. **Saving the old stack pointer**: The CPU reads the stack pointer (rsp) and stack segment (ss) register values and remembers them in an inernal buffer.
2. **Aligning the stack pointer**: An interrupt can occur at any instruction, so the stack pointer can have any value, too. However, some CPU instructions requires that the stack pointer be aligned on a 16-byte boundary, so the CPU performs such an alignment right after the interrupt.
3. **Switching stacks** (in some cases): A stack switch occurs when the CPU privilege level changes, for example, when a CPU exception occurs in a user-mode program. It is also possible to configure stack switches for specific interrupts using the so-called `Interrupt Stack Table`.
4. **Pushing the old stack pointer**: The CPU pushes the `rsp` and `ss` values from step 1 to the stack. This makes it possible to restore the original stack pointer when returning from an interrupt handler.

5. **Pushing and updating the `RFLAGS` register**: The `RFLAGS` register contains various control and status bits. On interrupt entry, the CPU changes some bits and pushes the old value.
6. **Pushing the instruction pointer**: Before jumping to the interupt handler function, the CPU pushes the unstruction pointer (`rip`) and the code segment (`cs`). This is comparable to the return address push of a normal function call.
7. **Pushing an error code** (for some exceptions): For some specific exceptions, such as page faults, the CPU pushes an error code, which describes the cause of the exception.
8. **Invoking the interrupt handler**: The CPU reads the address and the segment descriptor of the interrupt handler function from the corresponding field in the IDT. It then invokes this handler by loading values into the `rip` and `cs` registers.

In the `x86-64` crate, the interrupt stack frame is represented by the `InterruptStackFrame` struct. It is passed to interrupt handlers as `&mut` and can be used to retrieve additional infromation about the exception's cause. The struct contains no error code field, since only a few exceptions push an error code. These exceptions use the separate `HandlerFuncWithErrCode` function type, which has an addition `error_code` argument.

#### Behind the Scenes

The `x86-interrupt` calling convention is a powerful abstraction that hides almost all of the messy details of the exception handlers since we must not overwrite any register values before backing them up on stack. Here is a short overview of the things that the `x86-interrupt` calling convention takes care of:

1. **Retrieving the arguments**: Most calling conventions expect that the arguments are passed in registers. This is not possible for exception handlers since we must not overwrite any register value before backing them up on the stack. Instead, the `x86-interrupt` calling convention is aware that the arguments already lie on the stack at a specific offset.

2. **Returning using `iretq`**: Since the interrupt stack frame completely differs from stack frames of normal function calls, we can't return from handler functions through the normal `ret` instruction. So instead, the `iretq` instruction must be used.

3. **Handling the error code**: The error code, which is push for some exceptions , makes things much more complex. It changes the stack alignment and needs to be popped off the stack before returning. The `x86-interrupt` calling convention handles all that complexity. However, it doesn't know which handler function is used for which exception, so it needs to deduce that information from the number of function arguments. That means the programmer is still responsible for using the correct function type for each exception. Luckily, the `InterruptDescriptorTable` type defined by the `x86_64` crate ensures that the correct function types are used.

4. **Aligning the stack**: Some instructions require a 16-byte stack alignment. The CPU ensures this alignment whenever an exception occurs, but for some exceptions it destroys it again later when it pushes an error code. The x86-interrupt` calling convention takes care of this by realigning the stack in this case.

## Double Faults

Double fault occurs when the CPU fails to invoke an exception handler. By handling this exception, we avoid fatal `triple faults` that cause a system reset. To prevent triple faults in all cases, we set up an `Interrupt Stack Table` to catch double faults on a separte kernel stack.

### What is a Double Fault

Double fault is a special exception that occurs when the CPU dails to invoke and exception handler. For example, it happens when a page fault is triggered but there is no page fault handler registered in the `IDT`. So it's kind of similar to catch-all blocks in programming langyages with exception.

A double fault behaves like a normal exception. It has the vector number `8` and we can define a normal handler function for it in the IDT. It is really important to provide a double fault handler, because if a double fault is unhandled, a fatal `triple fault` occurs. This faults can't be caught, and most hardware reacts with a system reset.

### Cause of Double Fault
    A double fault is a special exception that occurs when the CPU fails to invoke and exception handler.
We always says this is the definition of double fault, but what does `fail to invoke` means? and what happens if a handler causes exceptions itself?

The AMD64 manual has an exact definition. Accroading to it, a **double fault exception can occur when a second exception occurs during the handling of a prior exception handler**. The `can` is important, it told us only very specific combinations of exceptions lead to a double fault. 

For example, a divide-by-zero fault followed by a page fault is fine, but a divide-by-zero fault followed by a general-protection fault leads to a double fault.

When the exception occurs, the CPU tries to read the corresponding IDT entry. Since the entry is 0, which is not a valid IDT entry, a `general protection fault` occurs. We did not define a handler function for the general protection fault either, so another general protection fault occurs. Accorading to the table, this leads to a double fault.

#### Kernel Stack Overflow
A guard page is a special memory page at the bottom of a stack that makes it possible to detect stack overflows. The page is not mapped to any physical frame, so accessing it causes a page fault instead of silently corrupting other memory. The bootloader sets up a guard page for out kernel stack, so a stack overflow causes a `page fault`.

When a page fault occurs, the CPU looks up the page fault handler in the IDT and tries to push the `interrupt stack frame` onto the stack. However, the current stack pointer still pointes to the non-present guard page. Thus, a second page fault occurs, which causes a double fault. 

So the CPU tries to call the double fault handler now
