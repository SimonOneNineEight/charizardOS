## Charizard - Operating System written in Rust

### Step 1. Create Bare Bones
#### Disable the Standard Library


However, we can still leverage other powerful implementation in rust like iterators, closires, pattern matching, and of course the ownership system. These features will enable us to build a kernel in a very expressive, high level way without worrying about undefined behavior or memory safety.

#### Panic Implementation
Rust handles panic in the standard library, which we disabled, so we will need to implement this necessary but missing feature.

#### The `eh_personality` Language Item
Language items are special functions and types that are required internally by the complier. For example, the `Copy` trait is a language item that tells the compiler which types have copy semantics. When we look at the implementation, we see it has the special `#[lang = "copy"]` attribute that defines it as a language item.

While providing custom implementation of language items is possible, it sould only be done as last resort, because that language items are highly unstable implementation details and not even type checked.

Fortunately, there is a more stable way to fix the above language error -- The `eh_personalily` language item, which marks a function that is used for implementing stact unwinding. But unfortunately this language also requres some OS-specific libraries, so we don't want to use it for our operating system and will disable it.

#### The `start` attribute
Although we might think that the `main` function as the first function called when we run our program. Most languages have `runtime system`, this system needs to be called before `main`, since it needs to initialize itself.

In a typical rust binary that links the standard library, execution starts in a C runtime library called `crt0`. which sets up the environment dor a C application, then the C runtime invokes the entry point of the Rust runtime, which is marked by the `start` language item. The runtime finally calls the `main` function.

Our OS executable doesn't hace access to the Rust runtime and crt0, s we need to define our own entry point. Implementing the `start` language item wouldn't help, since it would still require `crt0`. Instead, we need to overwrite `crt0` entry point directly.

#### Linker Error
The linker is a program that combines the generated code into an execuable. Since the executable format differs between different operating systems, and each of them throws a different error. The fundamental cause of the errors is the same: the default configuration of the linker auusmes that our program depends on the C runtime, which it does not.

To solve this error, we need to tell the linker that itshould not include the C runtime. We can do tis either by passing a certain set of arguments to the linker or by building for a bare metal target.

### Step2. A Minial Rust Kernel
#### Target Specification
We require some special configuration parameters for our OS, and there is no existing target triples fits, so we will have to define our own target through a JSON file.

Most of our config file will look similiar to the `x86_64-unknown-linux-gnu` target triple, but we set the `llvm-target` and `os` field, because we are running on bare metal. After that we changed our linker and move the panic-strategy from Cargo.toml to there. Finally we diable the redzone because we will need to handle interrupts at some time, and this is the way to do it safely. 

After completing our target file, we can finally successfullly build our 
operating system.

#### Printing the Screen
Finally, we can start to display something for our OS. The easiest way to print text to the screen at this stage is the VGA text buffer. It is a special memory area mapped to the VGA hardware that contains the contents display on screen. 

After we finish our simple display `Hello World!` code, we will need to turn our compiled Kernel into a bootable disk image by linking it with a bootloader. Then we can run the disk image in the QEMU virtual machine or boot it on real hardware using a USB stick.

To turn our compiled kernel into a bootable disk image, we need to link it with a bootloader, that is responsible for initializing the CPU and loading our kernel.


