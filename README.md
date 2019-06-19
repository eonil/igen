igen
====
Eonil

Generates RPC interfaces for other languages from Rust source code.
In other words, this is Rust language as IDL.

Supported Languages & Implementations
-------------------------------------
- Swift

Swift implementation uses `enum` to provide module-like
namespace.

Support for other languages will be added when I need it.

Design Choices
--------------
- This produces interfaces for out-of-process call.
  This means everything is passed by copy, and peers do not
  share anything.
  In-process calls are harder to design and stable.
  Maybe later.
- As it is out-of-process, everything must be serializable.
- Every intentions will be passed as messages.
- Designed only for bidirectional stream channels.
- Designed only for bidirectional message streams.
- There's no concept of "return values". 
  You have to design your protocol to work without waiting 
  for "return values". 
  This may sounds weird, but your protocol will be simpler
  and easier to maintain without "return values".
  See following chapters for details.
- As there's no "return value", all functions are encoded into
  `enum` variants.
- No explicit magic number.
- All strings are UTF-8 encoded.
- Uses JSON as default data encoding container. Other encodings
  can be added, but very unlikely.
- Rust code is "the standard schema". Encoded data will be generated
  based on Rust code. 
- Not all Rust data structures are supported. Only value-semantic
  and fully cloneable types will be supported.

"No"
----
- "return values".
- Concept of "function call". There's only "messages".
- Encoding of pointers or references. They're not serializable.
- Fixed sized array. 
  This is indistinguisable with dynamic sized array.

"Not Yet"
---------
- Versioning. Backward compatibility can be implicitly 
  implemented without magic numbers by diffing VCS history.

Designing Protocol without Return Values
----------------------------------------
This means all of your protocol must be designed in
fire & forget basis. Replies must encode its source
request so receivers can know how to apply the
replies.

Implementations
---------------
It's really hard to get fully qualified type paths from Rust
compiler. I'm not familiar with the compiler, and I abandoned
using compiler directly at some point. Helps are welcome.
Instead, `mgen` uses forked `rustdoc` code which already has
a complete type path resolution support. Due to dependencies of
`rustdoc`, whole Rust compiler is added as a dependency and
this is unavoidable until Rust compiler provide more convenient
facility to query fully qualified type paths.






License
-------
"MIT License". Contributions will become same license.




