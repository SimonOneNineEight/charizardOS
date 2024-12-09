### Implement Own Allocator
#### Why do we use `&self` instead of `&mut self` in `GlobalAlloc`
We need to use the `alloc` and `dealloc` function to mutate our `BumpAllocator` so that we can know where the next unused heap is. However, in the GlobAlloc trait, the parameter passed to those function is a immutable `&self`. This happens because we use the `#[global_allocator]` tag to static implements the `GlobalAlloc` trait, and a static variables is always immutable in Rust, so there is no way to call a method that takes `&mut self` on the static allocator.

### Why do we use `Bump Allocator`
The biggest advantage of bump allocation is that it's very fast compared to other allocator design that need to actively look for fitting memory block and perform various bookkeeping tasks on `alloc` and `dealloc`. This advantage can be useful for optimizing the allocation performance, for exampe when creating a `virtual DOM library`.

The main limitation of a bump allocator is that it can only reuse deallocated memory after all allocations have been freed. This means that a single long-lived allocation suffices to prevent memory reuse.

A bump allocator is seldonm used as a global allocator because of its drawback, but the principle of bump allocation is often applied in the form of `arena allocation`, which is basically batches individual allocations together to improve performance. kan example of an arena allocator for Rust is contained in the `toolshed` crate.


