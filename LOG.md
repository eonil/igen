Work Log
========
Eonil


2018/07/15
----------
I found this comment from `rustdoc` source code.

        // We need to hold on to the complete resolver, so we clone everything
        // for the analysis passes to use. Suboptimal, but necessary in the
        // current architecture.

It seems compiler erases all type mapping informations
after type resolution. Weird.


2018/07/15
----------
This program supercedes `cargo-ipcgen-swift`.
