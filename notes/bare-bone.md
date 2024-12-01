
# Charizard - Operating System written in Rust
## Part 1. Create Bare Bones
### Step 1. Setting up
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

### Step 2. A Minial Rust Kernel
#### Target Specification
We require some special configuration parameters for our OS, and there is no existing target triples fits, so we will have to define our own target through a JSON file.

Most of our config file will look similiar to the `x86_64-unknown-linux-gnu` target triple, but we set the `llvm-target` and `os` field, because we are running on bare metal. After that we changed our linker and move the panic-strategy from Cargo.toml to there. Finally we diable the redzone because we will need to handle interrupts at some time, and this is the way to do it safely. 

After completing our target file, we can finally successfullly build our 
operating system.

#### Printing the Screen
Finally, we can start to display something for our OS. The easiest way to print text to the screen at this stage is the VGA text buffer. It is a special memory area mapped to the VGA hardware that contains the contents display on screen. 

After we finish our simple display `Hello World!` code, we will need to turn our compiled Kernel into a bootable disk image by linking it with a bootloader. Then we can run the disk image in the QEMU virtual machine or boot it on real hardware using a USB stick.

To turn our compiled kernel into a bootable disk image, we need to link it with a bootloader, that is responsible for initializing the CPU and loading our kernel.

### Step 3. VGA text mode
The Vga text mode is a simple way to print text to the screen, and will be our primary way to display informations for our project. In this step, we are foing to create a interface that makes irs usage safe and simple by encapsulating all unsafety in a separate module, and will finally support the Rust formatting macros.

#### The VGA Text Buffer
We have to write the character into the text buffer of the VGA hardware if we want to print it on the screen in VGA mode. The VGA buffer is a two-dimensional array with typically 25 rows and 80 columns, which is directly render to the screen.

Each array entry describes a single screen character that is store in a `2 byte` memory through the following format:
|Bit(s)|Value|
|---|---|
|0-7|ASCII code|
|8-11|Foreground color|
|12-14|Background color|
|15| Blink|

The VGA text buffer is accessible via memory-mapped I/O to the address `0xb8000`. This means that reads and writes to that address don't access the RAM but directly access the text buffer on the VGA hardware. This means we can read and write it through normal memory operations to that address.

#### A Rust Module
We are going to create a Rust module to handle printing.
First, we create a `C-like` enum to explicitly specify the number for each color, and a struct to represent a full color code thatspecifies foreground and background color.

Following that, we need a buffer which is a 2-D array that has 25 rows and 80 columns that allows us to display the characters we want.

And finally, we create a write struct and implement the `write_byte` and `write_string` method to put values into our buffer. After these function created, we can now print characters on our os using VGA text buffer.

Although are able to print words on the screen, we only took care of the write part and haven't done the read part yet, which means that our complier doesn't know that we really access VGA buffer memory (instead of normal RAM) and knows nothing about the side effect that some characters apperar on the scree. So it might decide that these writes are unnecessary and can be omitted. To avoid this erronous optimization, we need to specify these writes as colatile. This tells the complier that the write has side effects and should not be optimized away. 

In order to use voltile writes for the VGA buffer, we use the `volatile` library. This crate provides a `Volatile` wratter type with `read` and `write` methods. These methods internally use the `read_volatile` and `write_volatile` functions of the core library and thus guarantee that the reads/writes are not optimized away.

Also, we created the `static Writer` as a `Global Interface` with `Lazy Statics` so that we are able to use the Writer in other modules without carrying that instance around.

To modify the value of our static Writer, we will need `Mutex` in the standard library. Unfortunately, we are not allowed to use the standard library, so we intead use the `spinlock` which doesn't requires using standard library and thread (standard Mutex uses thread to lock the resource) by the library `spin`.

After above settings, we are finally able to reproduce the `print` and `println` macro that we always use to display information on the screen.

### Step 4. Testing
Eventually, we arrived the last part of our bare metal OS -- Testing.

#### Testing in Rust
Rust has a built-in test framework that is capable of running unit test without the need to set anything up. But again, since we disabled the standard library, we can't use the Rust built-in test library because it depends on the standard library.

##### Custom Test Framework
Rust supports replacing the default test framework through the unstable `custom_test_framework` feature. This feature requires no external libraries and thus also works in `#![no_std]` environments. It collects all the functions annotated with a `#[test_case` attribute and the ninvoking a user-specified runner function with the list of tests as an argument. Thus, it gices the implementation maximal control over the test process.

The disadvantage compared to the default test framework is that many advanced features are not available. We will need to implement those features ourselves to complete the testing functionalities.

To do so, we will need to create the bare bone of our testing framework by implementing the `test_runner` function.

##### Exiting QEMU
After creating the test runner and servel mock test case, we will realize that we need to manually close the QEMU window after each test run, which is a unpleasant user experience, and violates the our goal which is to test without user interaction. 

Luckly, QEMU provided a `isa-debug-flag` that we can utilize to exit QEMU from guest system.

There are two different approaches for communicating between CPU and peripheral hardware on x86, `memory-mapped I/O` and `port-mapping I/O`. We already used memory-mapped I/O for accessing the VGA text buffer throughthe memory address `0xb8000`. This address is not mapped to RAM but to some memory on the VGA device.

In contrast, port-mapped I/O uses a separate I/O bus for communication. Each connected peripheral has one or more port numbers. To communicate with such an I/O port, there are special CPU instructions called `in` and `out`, which take a port number and a data byte. The `isa-debug-exit` device uses port-mapped I/O. The `iobase` parameter specifies on which port addess the device should liveand the `iosize` specifies the port size.

We create a `quit_qemu` function alone with a `QemuExitCode` enum that tells the test status and then put it into our test runner to complete the auto close feature of our QEMU virtual machine after running the test.

##### Hiding QEMU
To really seemlessly complete the test, we not only want to quit our kernel automatically, but don't even want a kernel window pops out when testing. This can also be beneficial when we runs our test on a platform without a user interface, such as CI/CD. To hide the QEMU kernel is relatively simple, we only need to pass the `-display none` when running the command, or we can put it in the `test-args` array in the `Cargo.toml` file.

##### Timeouts
Since `cargo test` waits until the test runner exits, a test that never returns can block the test runner forever, this won't be a problem in practice since it's usually easy to avoid the endless loops. However, endless loops can occur in carious situation in our case. For example: The bootloader failed to load the kernel.

Because endless loop can occur in so many situations, the `bootimage` tool we use set a timeout of 5 minutes for each test executable by default. If the test doesn't finish within this time, it is marked as dailed and a **"Time Out"** error is printed to the console. And we are able to adjust the test duration with the `test-timeout` attribute under the `[package.metadata.bootimag]` section in our `Cargo.toml` file.

#### Printing to the Console
Next important implementation for our test framework is to display print the test result to our console. We've used the VGA text buffer to display informations on our QEMU virtual machine, but It only remains in that QEMU machine, and we are not able to get those data from our console. That is why we will need to find a supplement method, which is `serial port` -- a simple way to send data from our QEMU kernel to the host's standard output or a file.

The `serial_print` is pretty much like our `print` macro, which is make a stactic global interface, and then a `_print` function that mutates the underlying value, and finally create the `serial_print` and `serial_println` macro.

#### Intergration Test
The convention for `intergration tests` in Rust is to put them into a `test` directory next to the `src` directory. Both the default test framework and custom test frameworks will automatically pick up and execute all tests in that directory.

We also create a library that move the required functions available to our intergration test. 

The power of intergration texts is that they're treated as completely separate executable. This gives them complete control over the environment, which make it possible to test that the tcode interacts correctly with the CPU or hardware messages.

By adding tests, we can ensure that we don't break our built feature when we add new implementations to out kernel or refactor our code. This is espiecally important when our kernel becomes larger and more conplex.
