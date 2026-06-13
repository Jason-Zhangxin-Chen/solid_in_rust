// In Rust, a fat pointer is a two‑word pointer that bundles a raw address with additional
// metadata needed to handle dynamically sized types (DSTs).

// Both Box<[T]> and Box<dyn Trait> are fat pointers stored on the stack, but the metadata
// they carry—and what they point to on the heap—differs.

// Box<[i32]> - pointer + length.
// Stack (Box itself):
// ┌───────────────────┐
// │  ptr to elements  │  ( *mut i32, 8 bytes on 64‑bit)
// ├───────────────────┤
// │  length (usize)   │  (8 bytes)
// └───────────────────┘
//          │
// Heap:    ▼
// ┌────────┬────────┬───┬────────┐
// │ i32[0] │ i32[1] │ … │ i32[n] │
// └────────┴────────┴───┴────────┘

// The Box is two words: a data pointer to the first i32 and a usize length.
//
// On the heap you find the actual contiguous i32 array (no extra header).
//
// The length lives only inside the fat pointer, not on the heap.
//
// When you coerce a Box<[i32; N]> into a Box<[i32]>, the pointer’s address stays the same, and the
// length becomes N.
//
// Similarly, Box<str> is a fat pointer with a data pointer to the UTF‑8 bytes and a length.


// Box<dyn Trait> – pointer + vtable pointer
// Stack (Box itself):
// ┌───────────────────┐
// │  data pointer     │  ( *const (), 8 bytes)
// ├───────────────────┤
// │  vtable pointer   │  ( &'static VTable, 8 bytes)
// └───────────────────┘
//          │
// Heap:    ▼
// ┌─────────────────────┐
// │  concrete type's    │
// │  data (T)           │
// └─────────────────────┘
//          │
//          │         Static vtable (read‑only data section)
//          └────────► ┌───────────────────┐
//                     │ size              │ (usize)
//                     │ alignment         │ (usize)
//                     │ drop fn           │ (fn pointer)
//                     │ trait method #1   │ (fn pointer)
//                     │ trait method #2   │ (fn pointer)
//                     │ …                 │
//                     └───────────────────┘
// The Box holds two pointers:
//
// data pointer → a thin pointer to the heap‑allocated value (the concrete type T).
//
// vtable pointer → a &'static VTable that describes the erased type’s layout and the
// implementations of the trait’s methods.
//
// On the heap: only the concrete value T (exactly as it was allocated) – no additional wrapper
// or header.
//
// The vtable is a static structure; every concrete type T: Trait has its own vtable. It contains:
//
// size and alignment of T (for deallocation / Layout).
//
// A drop function pointer (drop_in_place::<T>).
//
// Function pointers for all methods of the trait (including supertraits).
//
// When you call obj.method(), the compiler reads the correct function pointer from the vtable
// and calls it with the data pointer as the self argument. This is dynamic dispatch
// (one level of indirection).